use std::collections::VecDeque;
use std::fmt;
use std::thread;
use std::rc::Rc;
use std::sync::mpsc;

use glium::{self, DisplayBuild, Surface};

use cpu;
use device::{keyboard, lem1802};

enum ThreadCommand {
    Stop,
}

enum KeyboardEvent {
    KeyPressed(keyboard::Key),
    KeyReleased(keyboard::Key),
}

struct CommonBackend {
    thread_handle: Option<thread::JoinHandle<()>>,
    thread_command: mpsc::Sender<ThreadCommand>,
}

pub struct KeyboardBackend {
    // used for Drop
    #[allow(dead_code)]
    common: Rc<CommonBackend>,
    keyboard_receiver: mpsc::Receiver<KeyboardEvent>,
    key_pressed: [bool; 0x92],
}

pub struct ScreenBackend {
    // used for Drop
    #[allow(dead_code)]
    common: Rc<CommonBackend>,
    screen_sender: mpsc::Sender<Box<lem1802::Screen>>,
}

pub fn start() -> (ScreenBackend, KeyboardBackend) {
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();
    let (tx3, rx3) = mpsc::channel();
    let handle = thread::spawn(move || thread_main(rx1, tx2, rx3));
    let common = Rc::new(CommonBackend {
        thread_handle: Some(handle),
        thread_command: tx1,
    });
    (ScreenBackend {
        common: common.clone(),
        screen_sender: tx3,
    },KeyboardBackend {
        common: common,
        keyboard_receiver: rx2,
        key_pressed: [false; 0x92],
    })
}

impl fmt::Debug for KeyboardBackend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Glium backend")
    }
}

impl keyboard::Backend for KeyboardBackend {
    fn is_key_pressed(&mut self, key: keyboard::Key) -> bool {
        self.key_pressed[key.encode() as usize]
    }

    fn push_typed_keys(&mut self, queue: &mut VecDeque<keyboard::Key>) -> bool {
        let mut new_keys = false;
        loop {
            match self.keyboard_receiver.try_recv() {
                Ok(KeyboardEvent::KeyPressed(k)) => {
                    new_keys = true;
                    self.key_pressed[k.encode() as usize] = true;
                    queue.push_back(k);
                }
                Ok(KeyboardEvent::KeyReleased(k)) => {
                    new_keys = true;
                    self.key_pressed[k.encode() as usize] = false;
                }
                Err(mpsc::TryRecvError::Empty) => return new_keys,
                Err(mpsc::TryRecvError::Disconnected) => panic!("Thread down"),
            }
        }
    }
}

impl fmt::Debug for ScreenBackend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Glium backend")
    }
}

impl lem1802::Backend for ScreenBackend {
    fn tick<B: lem1802::Backend>(&self,
                                 cpu: &cpu::Cpu,
                                 lem: &lem1802::LEM1802<B>,
                                 tick_count: u64) {
        // TODO: 10 fps for now by fear to fill the buffer
        if tick_count % 10_000 == 0 {
            self.screen_sender.send(lem.get_screen(cpu)).unwrap();
        }
    }
}

impl Drop for CommonBackend {
    fn drop(&mut self) {
        if let Some(handle) = self.thread_handle.take() {
            let _ = self.thread_command.send(ThreadCommand::Stop);
            handle.join().unwrap();
        }
    }
}

fn thread_main(thread_command: mpsc::Receiver<ThreadCommand>,
               keyboard_sender: mpsc::Sender<KeyboardEvent>,
               screen_receiver: mpsc::Receiver<Box<lem1802::Screen>>) {
    let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();
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
        #version 140
        #define SCREEN_WIDTH 128.
        #define SCREEN_HEIGHT 96.

        in uint i;
        in uint j;
        in vec3 color;
        in vec2 position;

        out vec3 v_color;

        uniform float aspect_ratio;

        void main() {
            mat4 rot = mat4(
                vec4(0, 1, 0, 0),
                vec4(-aspect_ratio, 0, 0, 0),
                vec4(0, 0, 1, 0),
                vec4(0, 0, 0, 1)
            );
            v_color = color;
            gl_Position = rot * vec4(
                ((position[0] + float(j)) / SCREEN_WIDTH * 2.) - 1.,
                ((position[1] + float(i)) / SCREEN_HEIGHT * 2.) - 1.,
                0.0,
                1.0
            );
        }
    ", "
        #version 140
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
                Ok(screen) => {
                    current_screen = screen;
                    break 'pote2;
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
        target.draw(
            (&vertex_buffer, per_instance.per_instance().unwrap()),
            &indices,
            &program,
            &uniform! { aspect_ratio: aspect_ratio },
            &Default::default()
        ).unwrap();
        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => break 'main,
                glium::glutin::Event::KeyboardInput(state, raw, code) => {
                    if let Some(code) = code {
                        if let Some(converted) = convert_kb_code(raw, code) {
                            keyboard_sender.send(match state {
                                glium::glutin::ElementState::Pressed =>
                                    KeyboardEvent::KeyPressed(converted),
                                glium::glutin::ElementState::Released =>
                                    KeyboardEvent::KeyReleased(converted)
                            }).unwrap();
                        }
                    }
                }
                _ => ()
            }
        }

        'pote: loop {
            match thread_command.try_recv() {
                Ok(ThreadCommand::Stop) => break 'main,
                Err(mpsc::TryRecvError::Empty) => break 'pote,
                Err(mpsc::TryRecvError::Disconnected) => break 'main,
            }
        }
    }
}

fn convert_kb_code(raw: u8,
                   code: glium::glutin::VirtualKeyCode)
    -> Option<keyboard::Key> {
    use glium::glutin::VirtualKeyCode;
    use device::keyboard::Key;
    match code {
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
        _ if 0x20 <= raw && raw <= 0x7f => Some(Key::ASCII(raw as u16)),
        _ => None,
    }
}
