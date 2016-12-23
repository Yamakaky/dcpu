use std::thread;
use std::sync::{mpsc, Arc, Mutex};

use glium::{self, DisplayBuild, Surface};

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
        CreationError(glium::GliumCreationError<glium::glutin::CreationError>);
        VertexCreationError(glium::vertex::BufferCreationError);
        ProgramCreationError(glium::ProgramCreationError);
        SwapBuffersError(glium::SwapBuffersError);
        DrawError(glium::DrawError);
        MpscSendError(mpsc::SendError<KeyboardEvent>);
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
    let mut current_screen =
        Box::new(lem1802::Screen([lem1802::Color::default(); 12288]));

    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
        }
        implement_vertex!(Vertex, position);

        let data = [
            Vertex { position: [0., 0.]},
            Vertex { position: [0., 1.]},
            Vertex { position: [1., 0.]},
            Vertex { position: [1., 1.]},
        ];
        try!(glium::vertex::VertexBuffer::new(&display, &data))
    };

    let mut per_instance = {
        #[derive(Debug, Copy, Clone)]
        struct Attr {
            i: u16,
            j: u16,
            color: [f32; 3],
        }
        implement_vertex!(Attr, i, j, color);
        let mut data = Vec::with_capacity((lem1802::SCREEN_WIDTH * lem1802::SCREEN_HEIGHT) as usize);
        for j in 0..lem1802::SCREEN_HEIGHT {
            for i in 0..lem1802::SCREEN_WIDTH {
                data.push(Attr {
                    i: i,
                    j: j,
                    color: [0.; 3],
                })
            }
        }
        try!(glium::vertex::VertexBuffer::dynamic(&display, &data))
    };
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

    let program = try!(glium::Program::from_source(&display, "
        #version 130
        #define SCREEN_WIDTH 128.
        #define SCREEN_HEIGHT 96.

        in uint i;
        in uint j;
        in vec3 color;
        in vec2 position;

        out vec3 v_color;

        uniform float aspect_ratio;

        void main() {
            v_color = color;
            gl_Position = vec4(
                ((position[0] + float(i)) / SCREEN_WIDTH - 0.5) * 2.,
                (-(position[1] + float(j)) / SCREEN_HEIGHT + 0.5) * 2.,
                0.,
                1.
            );
        }
    ", "
        #version 130
        in vec3 v_color;
        out vec4 f_color;

        void main() {
            f_color = vec4(v_color, 1.0);
        }
    ",
    None));

    'main: loop {
        'pote2: loop {
            match screen_receiver.try_recv() {
                Ok(ScreenCommand::Show(screen)) => {
                    current_screen = screen.into();
                    display.get_window().map(|w| w.show());
                }
                Ok(ScreenCommand::Hide) => {
                    display.get_window().map(|w| w.hide());
                }
                Err(mpsc::TryRecvError::Empty) => break 'pote2,
                Err(mpsc::TryRecvError::Disconnected) => break 'main,
            }
        }

        {
            let mut mapping = per_instance.map();
            for (color, dst) in current_screen.0
                                              .iter()
                                              .zip(mapping.iter_mut()) {
                dst.color = [color.r, color.g, color.b];
            }
        }

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        let aspect_ratio = {
            let (width, height) = target.get_dimensions();
            height as f32 / width as f32
        };
        try!(target.draw((&vertex_buffer, per_instance.per_instance().unwrap()),
                         &indices,
                         &program,
                         &uniform! { aspect_ratio: aspect_ratio },
                         &Default::default()));
        try!(target.finish());

        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => break 'main,
                glium::glutin::Event::KeyboardInput(state, raw, code) => {
                    if let Some(converted) = convert_kb_code(raw, code) {
                        try!(keyboard_sender.send(match state {
                            glium::glutin::ElementState::Pressed =>
                                KeyboardEvent::KeyPressed(converted),
                            glium::glutin::ElementState::Released =>
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

fn convert_kb_code(raw: u8, maybe_code: Option<glium::glutin::VirtualKeyCode>)
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
