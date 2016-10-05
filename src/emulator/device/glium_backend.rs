use std::thread;
use std::rc::Rc;
use std::sync::mpsc;

use glium::{self, DisplayBuild, Surface};

use emulator::device::{keyboard, lem1802};
use emulator::device::keyboard::mpsc_backend::*;
use emulator::device::lem1802::generic_backend::*;

enum ThreadCommand {
    Stop,
}

struct CommonBackend {
    thread_handle: Option<thread::JoinHandle<()>>,
    thread_command: mpsc::Sender<ThreadCommand>,
}

impl Drop for CommonBackend {
    fn drop(&mut self) {
        if let Some(handle) = self.thread_handle.take() {
            let _ = self.thread_command.send(ThreadCommand::Stop);
            handle.join().unwrap();
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
    let common = Rc::new(CommonBackend {
        thread_handle: Some(handle),
        thread_command: tx1,
    });
    (ScreenBackend::new(common.clone(), move |s| tx3.send(s).unwrap()),
     KeyboardBackend::new(common, rx2))
}

fn thread_main(thread_command: mpsc::Receiver<ThreadCommand>,
               keyboard_sender: mpsc::Sender<KeyboardEvent>,
               screen_receiver: mpsc::Receiver<ScreenCommand>) {
    let display = glium::glutin::WindowBuilder::new()
        .with_title("Screen + keyboard")
        .with_vsync()
        .with_visibility(false)
        .build_glium()
        .unwrap();
    let mut current_screen = Box::new([lem1802::Color::default(); 12288]);

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
        glium::vertex::VertexBuffer::new(&display, &data).unwrap()
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
        glium::vertex::VertexBuffer::dynamic(&display, &data).unwrap()
    };
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

    let program = glium::Program::from_source(&display, "
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
    None).unwrap();

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

        {
            let mut mapping = per_instance.map();
            for (color, dst) in current_screen.iter().zip(mapping.iter_mut()) {
                dst.color = [color.r, color.g, color.b];
            }
        }

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        let aspect_ratio = {
            let (width, height) = target.get_dimensions();
            height as f32 / width as f32
        };
        target.draw((&vertex_buffer, per_instance.per_instance().unwrap()),
                    &indices,
                    &program,
                    &uniform! { aspect_ratio: aspect_ratio },
                    &Default::default())
              .unwrap();
        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => break 'main,
                glium::glutin::Event::KeyboardInput(state, raw, code) => {
                    if let Some(converted) = convert_kb_code(raw, code) {
                        keyboard_sender.send(match state {
                            glium::glutin::ElementState::Pressed =>
                                KeyboardEvent::KeyPressed(converted),
                            glium::glutin::ElementState::Released =>
                                KeyboardEvent::KeyReleased(converted)
                        }).unwrap();
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
