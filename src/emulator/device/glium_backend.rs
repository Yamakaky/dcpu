use std::thread;
use std::sync::{mpsc, Arc, Mutex};

use glium::{self, glutin, index, framebuffer, texture, vertex};
use glium::{DisplayBuild, Surface, Program};
use glium::uniforms::UniformBuffer;

use emulator::device::{self, keyboard, lem1802};
use emulator::device::keyboard::mpsc_backend::*;
use emulator::device::lem1802::generic_backend::*;

enum ThreadCommand {
    Stop,
}

struct CommonBackend {
    thread_handle: Option<thread::JoinHandle<Result<()>>>,
    thread_command: mpsc::Sender<ThreadCommand>,
}

impl Drop for CommonBackend {
    fn drop(&mut self) {
        if let Some(handle) = self.thread_handle.take() {
            let _ = self.thread_command.send(ThreadCommand::Stop);
            match handle.join() {
                Ok(res) => if let Err(e) = res {
                    println!("Glium backend error: {}", e);
                    if let Some(backtrace) = e.backtrace() {
                        println!("{:?}", backtrace);
                    }
                },
                Err(_) => unimplemented!(),
            }
        }
    }
}

pub fn start() -> (ScreenBackend, KeyboardBackend) {
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();
    let (tx3, rx3) = mpsc::channel();
    let handle = thread::Builder::new()
        .name("glium".into())
        .spawn(move || thread_main(rx1, tx2, rx3))
        .unwrap();
    let common = Arc::new(Mutex::new(CommonBackend {
        thread_handle: Some(handle),
        thread_command: tx1,
    }));
    let callback = move |s| {
        tx3.send(s).map_err(|_| {
            device::ErrorKind::BackendStopped("glium".into()).into()
        })
    };
    (ScreenBackend::new(common.clone(), callback),
     KeyboardBackend::new(common, rx2))
}

error_chain! {
    foreign_links {
        glium::GliumCreationError<glium::glutin::CreationError>, CreationError;
        framebuffer::ValidationError, ValidationError;
        glium::buffer::BufferCreationError, BufferCreationError;
        vertex::BufferCreationError, VertexCreationError;
        texture::TextureCreationError, TextureCreationError;
        glium::ProgramCreationError, ProgramCreationError;
        glium::SwapBuffersError, SwapBuffersError;
        glium::DrawError, DrawError;
        mpsc::SendError<KeyboardEvent>, MpscSendError;
    }
}

fn thread_main(thread_command: mpsc::Receiver<ThreadCommand>,
               keyboard_sender: mpsc::Sender<KeyboardEvent>,
               screen_receiver: mpsc::Receiver<ScreenCommand>)
    -> Result<()> {
    let display = try!(glium::glutin::WindowBuilder::new()
        .with_title("Screen + keyboard")
        .with_vsync()
        .with_visibility(false)
        .build_glium());
    let mut current_screen = Box::new(lem1802::RawScreen {
        vram: lem1802::Vram([0; 386]),
        font: lem1802::Font([0; 256]),
        palette: [0; 16],
        border_color_index: 0,
    });

    let render_vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
        }
        implement_vertex!(Vertex, position);

        let data = [
            Vertex { position: [-1., -1.]},
            Vertex { position: [-1., 1.]},
            Vertex { position: [1., -1.]},
            Vertex { position: [1., 1.]},
        ];
        try!(vertex::VertexBuffer::immutable(&display, &data))
    };

    let composition_vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            texcoord: [f32; 2],
        }
        implement_vertex!(Vertex, position, texcoord);

        let data = [
            Vertex { position: [-0.9, -0.9], texcoord: [0., 0.] },
            Vertex { position: [-0.9, 0.9], texcoord: [0., 1.] },
            Vertex { position: [0.9, -0.9], texcoord: [1., 0.] },
            Vertex { position: [0.9, 0.9], texcoord: [1., 1.] },
        ];
        try!(vertex::VertexBuffer::dynamic(&display, &data))
    };

    let vram: UniformBuffer<[u32; 512]> =
        try!(UniformBuffer::empty_dynamic(&display));
    let font: UniformBuffer<[u32; 256]> =
        try!(UniformBuffer::empty_dynamic(&display));
    let palette: UniformBuffer<[u32; 16]> =
        try!(UniformBuffer::empty_dynamic(&display));
    let render_buffer = try!(texture::Texture2d::empty_with_format(
        &display,
        texture::UncompressedFloatFormat::F32F32F32F32,
        texture::MipmapsOption::NoMipmap,
        128,
        96));
    let mut frame_buffer =
        try!(framebuffer::SimpleFrameBuffer::new(&display, &render_buffer));

    let indices = index::NoIndices(index::PrimitiveType::TriangleStrip);

    let render_program = try!(Program::from_source(&display, r#"
        #version 330

        in vec2 position;

        void main() {
            gl_Position = vec4(position.xy, 0., 1.);
        }
    "#, r#"
        #version 330

        const uint MASK_INDEX = 0xfu;
        const uint SCREEN_HEIGHT = 96u;
        const uint SCREEN_WIDTH = 128u;
        const uint SCREEN_SIZE = SCREEN_WIDTH * SCREEN_HEIGHT;
        const uint CHAR_HEIGHT = 8u;
        const uint CHAR_WIDTH = 4u;
        const uint CHAR_SIZE = CHAR_HEIGHT * CHAR_WIDTH;
        const uint NB_CHARS = 32u * 12u;

        const uint MASK_BLINKING = 1u << 7u;
        const uint MASK_COLOR_IDX = 0xfu;
        const uint MASK_CHAR = 0x7fu;
        const uint SHIFT_FG = 12u;
        const uint SHIFT_BG = 8u;

        layout(origin_upper_left, pixel_center_integer) in vec4 gl_FragCoord;

        uniform Vram {
            uint vram[512];
        };
        uniform Font {
            uint font[256];
        };
        uniform Palette {
            uint palette[16];
        };

        out vec4 f_color;

        struct VideoWord {
            uint char_idx;
            uint bg_idx;
            uint fg_idx;
            bool blinking;
        };

        VideoWord vw_from_packed(uint w) {
            return VideoWord(w & MASK_CHAR,
                             (w >> SHIFT_BG) & MASK_COLOR_IDX,
                             (w >> SHIFT_FG) & MASK_COLOR_IDX,
                             (w & MASK_BLINKING) != 0u);
        }

        uint get_font(uint char_idx) {
            uint w0 = font[char_idx * 2u];
            uint w1 = font[char_idx * 2u + 1u];
            return (w0 << 16u) | w1;
        }

        vec4 get_color(uint color_idx) {
            uint c = palette[color_idx];
            return vec4(float((c >> 8u) & 0xfu) / 15.0,
                        float((c >> 4u) & 0xfu) / 15.0,
                        float(c & 0xfu) / 15.0,
                        1.0);
        }

        void main() {
            uint char_offset = uint(gl_FragCoord.x) / CHAR_WIDTH + (uint(gl_FragCoord.y) / CHAR_HEIGHT) * (SCREEN_WIDTH / CHAR_WIDTH);
            VideoWord video_word = vw_from_packed(vram[char_offset]);
            uint font_item = get_font(video_word.char_idx);
            uint x = uint(gl_FragCoord.x) % CHAR_WIDTH;
            uint y = uint(gl_FragCoord.y) % CHAR_HEIGHT;
            uint bit = (font_item >> (x * CHAR_HEIGHT + 7u - y)) & 1u;
            if (bit == 0u) {
                f_color = get_color(video_word.bg_idx);
            } else {
                f_color = get_color(video_word.fg_idx);
            }
        }
    "#,
    None));

    let composition_program = try!(Program::from_source(&display, r#"
        #version 330

        in vec2 position;
        in vec2 texcoord;

        smooth out vec2 v_texcoord;

        void main() {
            v_texcoord = texcoord;
            gl_Position = vec4(position.xy, 0., 1.);
        }
    "#, r#"
        #version 330

        smooth in vec2 v_texcoord;

        uniform sampler2D screen_texture;

        out vec4 f_color;

        void main() {
            f_color = vec4(texture(screen_texture, v_texcoord).rgb, 1.0);
        }
    "#,
    None));

    'main: loop {
        'pote2: loop {
            match screen_receiver.try_recv() {
                Ok(ScreenCommand::Show(screen)) => {
                    current_screen = screen;
                    display.get_window().map(|w| w.show());
                }
                Ok(ScreenCommand::Hide) => {
                    display.get_window().map(|w| w.hide());
                }
                Err(mpsc::TryRecvError::Empty) => break 'pote2,
                Err(mpsc::TryRecvError::Disconnected) => break 'main,
            }
        }

        let mut vram_data = [0; 512];
        for (from, to) in current_screen.vram.0.iter().zip(vram_data.iter_mut()) {
            *to = *from as u32;
        }
        vram.write(&vram_data);
        let mut font_data = [0; 256];
        for (from, to) in current_screen.font.0.iter().zip(font_data.iter_mut()) {
            *to = *from as u32;
        }
        font.write(&font_data);
        let mut palette_data = [0; 16];
        for (from, to) in current_screen.palette.iter().zip(palette_data.iter_mut()) {
            *to = *from as u32;
        }
        palette.write(&palette_data);

        frame_buffer.clear_color(0.0, 0.0, 1.0, 1.0);
        try!(frame_buffer.draw(&render_vertex_buffer,
                               &indices,
                               &render_program,
                               &uniform! {
                                   Vram: &vram,
                                   Font: &font,
                                   Palette: &palette,
                               },
                               &Default::default()));

        let mut target = display.draw();
        let border_color =
            current_screen.get_color(current_screen.border_color_index);
        target.clear_color(border_color.r,
                           border_color.g,
                           border_color.b,
                           1.0);
        let aspect_ratio = {
            let (width, height) = target.get_dimensions();
            height as f32 / width as f32
        };
        try!(target.draw(&composition_vertex_buffer,
                         &indices,
                         &composition_program,
                         &uniform! {
                             aspect_ratio: aspect_ratio,
                             screen_texture: &render_buffer,
                         },
                         &Default::default()));
        try!(target.finish());

        for ev in display.poll_events() {
            match ev {
                glutin::Event::Closed => break 'main,
                glutin::Event::KeyboardInput(state, raw, code) => {
                    if let Some(converted) = convert_kb_code(raw, code) {
                        try!(keyboard_sender.send(match state {
                            glutin::ElementState::Pressed =>
                                KeyboardEvent::KeyPressed(converted),
                            glutin::ElementState::Released =>
                                KeyboardEvent::KeyReleased(converted)
                        }));
                    }
                }
                _ => ()
            }
        }

        'pote: loop {
            match thread_command.try_recv() {
                Ok(ThreadCommand::Stop) |
                Err(mpsc::TryRecvError::Disconnected) => break 'main,
                Err(mpsc::TryRecvError::Empty) => break 'pote,
            }
        }
    }

    // For some reason, the window is not closed with the end of the thread
    display.get_window().map(|w| w.hide());
    Ok(())
}

fn convert_kb_code(raw: u8, maybe_code: Option<glutin::VirtualKeyCode>)
    -> Option<keyboard::Key> {
    use glium::glutin::VirtualKeyCode;
    use emulator::device::keyboard::Key;
    maybe_code.and_then(|code| match code {
        VirtualKeyCode::Back => Some(Key::Backspace),
        VirtualKeyCode::Return => Some(Key::Return),
        VirtualKeyCode::Insert => Some(Key::Insert),
        VirtualKeyCode::Delete => Some(Key::Delete),
        VirtualKeyCode::Up => Some(Key::Up),
        VirtualKeyCode::Down => Some(Key::Down),
        VirtualKeyCode::Left => Some(Key::Left),
        VirtualKeyCode::Right => Some(Key::Right),
        VirtualKeyCode::LShift | VirtualKeyCode::RShift => Some(Key::Shift),
        VirtualKeyCode::LControl | VirtualKeyCode::RControl=> Some(Key::Control),
        VirtualKeyCode::Key1 => Some(Key::ASCII('1' as u16)),
        VirtualKeyCode::Key2 => Some(Key::ASCII('2' as u16)),
        VirtualKeyCode::Key3 => Some(Key::ASCII('3' as u16)),
        VirtualKeyCode::Key4 => Some(Key::ASCII('4' as u16)),
        VirtualKeyCode::Key5 => Some(Key::ASCII('5' as u16)),
        VirtualKeyCode::Key6 => Some(Key::ASCII('6' as u16)),
        VirtualKeyCode::Key7 => Some(Key::ASCII('7' as u16)),
        VirtualKeyCode::Key8 => Some(Key::ASCII('8' as u16)),
        VirtualKeyCode::Key9 => Some(Key::ASCII('9' as u16)),
        VirtualKeyCode::Key0 => Some(Key::ASCII('0' as u16)),
        VirtualKeyCode::A => Some(Key::ASCII('a' as u16)),
        VirtualKeyCode::B => Some(Key::ASCII('b' as u16)),
        VirtualKeyCode::C => Some(Key::ASCII('c' as u16)),
        VirtualKeyCode::D => Some(Key::ASCII('d' as u16)),
        VirtualKeyCode::E => Some(Key::ASCII('e' as u16)),
        VirtualKeyCode::F => Some(Key::ASCII('f' as u16)),
        VirtualKeyCode::G => Some(Key::ASCII('g' as u16)),
        VirtualKeyCode::H => Some(Key::ASCII('h' as u16)),
        VirtualKeyCode::I => Some(Key::ASCII('i' as u16)),
        VirtualKeyCode::J => Some(Key::ASCII('j' as u16)),
        VirtualKeyCode::K => Some(Key::ASCII('k' as u16)),
        VirtualKeyCode::L => Some(Key::ASCII('l' as u16)),
        VirtualKeyCode::M => Some(Key::ASCII('m' as u16)),
        VirtualKeyCode::N => Some(Key::ASCII('n' as u16)),
        VirtualKeyCode::O => Some(Key::ASCII('o' as u16)),
        VirtualKeyCode::P => Some(Key::ASCII('p' as u16)),
        VirtualKeyCode::Q => Some(Key::ASCII('q' as u16)),
        VirtualKeyCode::R => Some(Key::ASCII('r' as u16)),
        VirtualKeyCode::S => Some(Key::ASCII('s' as u16)),
        VirtualKeyCode::T => Some(Key::ASCII('t' as u16)),
        VirtualKeyCode::U => Some(Key::ASCII('u' as u16)),
        VirtualKeyCode::V => Some(Key::ASCII('v' as u16)),
        VirtualKeyCode::W => Some(Key::ASCII('w' as u16)),
        VirtualKeyCode::X => Some(Key::ASCII('x' as u16)),
        VirtualKeyCode::Y => Some(Key::ASCII('y' as u16)),
        VirtualKeyCode::Z => Some(Key::ASCII('z' as u16)),
        VirtualKeyCode::Space => Some(Key::ASCII(' ' as u16)),
        _ => None,
    }).or(match raw {
            // 0-9
            19 => Some(Key::ASCII('0' as u16)),
            x if 10 <= x && x <= 18 =>
                Some(Key::ASCII('1' as u16 + x as u16 - 10)),
            _ => None,
    })
}
