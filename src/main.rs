use clap::{Parser, Subcommand};
use enigo::{Enigo, KeyboardControllable};
use std::io::{Read, Write};
use std::net::{IpAddr, TcpListener, TcpStream};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const STATE_PRESSED: u8 = 0;
const STATE_RELEASED: u8 = 1;

const MODE_CHAR: u8 = 0;
const MODE_CTRL: u8 = 1;
const MODE_SHIFT: u8 = 2;
const MODE_ALT: u8 = 3;
const MODE_BACKSPACE: u8 = 4;
const MODE_CAPS_LOCK: u8 = 5;
const MODE_DELETE: u8 = 6;
const MODE_DOWN: u8 = 7;
const MODE_UP: u8 = 8;
const MODE_LEFT: u8 = 9;
const MODE_RIGHT: u8 = 10;
const MODE_END: u8 = 11;
const MODE_ESCAPE: u8 = 12;
const MODE_F1: u8 = 13;
const MODE_F2: u8 = 14;
const MODE_F3: u8 = 15;
const MODE_F4: u8 = 16;
const MODE_F5: u8 = 17;
const MODE_F6: u8 = 18;
const MODE_F7: u8 = 19;
const MODE_F8: u8 = 20;
const MODE_F9: u8 = 21;
const MODE_F10: u8 = 22;
const MODE_F11: u8 = 23;
const MODE_F12: u8 = 24;
const MODE_F13: u8 = 25;
const MODE_F14: u8 = 26;
const MODE_F15: u8 = 27;
const MODE_F16: u8 = 28;
const MODE_F17: u8 = 29;
const MODE_F18: u8 = 30;
const MODE_F19: u8 = 31;
const MODE_F20: u8 = 32;
const MODE_HELP: u8 = 33;
const MODE_HOME: u8 = 34;
const MODE_META: u8 = 35;
const MODE_OPTION: u8 = 36;
const MODE_PAGE_DOWN: u8 = 37;
const MODE_PAGE_UP: u8 = 38;
const MODE_RETURN: u8 = 39;
const MODE_SPACE: u8 = 40;
const MODE_TAB: u8 = 41;

#[derive(Debug, Parser)]
#[command(name = "remote-keyboard")]
#[command(about = "Remote controlling key presses", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: ArgCommand,
}

#[derive(Debug, Subcommand)]
enum ArgCommand {
    /// Hosts keyboard control, allowing your computer to be controlled
    Host {
        /// The port to host on
        #[arg(short, long, default_value_t = 58008)]
        port: u16,
    },
    /// Connect to someone else's computer
    #[command(arg_required_else_help = true)]
    Connect {
        /// The IP to connect to
        ip: IpAddr,
        /// The port to connect to
        #[arg(short, long, default_value_t = 58008)]
        port: u16,
    },
}

fn main() {
    let args = Args::parse();
    match args.command {
        ArgCommand::Host { port } => host(port),
        ArgCommand::Connect { ip, port } => connect(ip, port),
    }
}

fn host(port: u16) {
    let listener = match TcpListener::bind(("0.0.0.0", port)) {
        Ok(listener) => listener,
        Err(err) => {
            println!("{err}");
            return;
        }
    };

    println!("Listening on port {port}...");

    let stream = listener.incoming().next().unwrap();

    println!("Received client");

    let mut stream = match stream {
        Ok(stream) => stream,
        Err(err) => {
            println!("{err}");
            return;
        }
    };

    let mut enigo = Enigo::new();
    loop {
        if let Err(err) = process_event(&mut stream, &mut enigo) {
            if err.kind() == std::io::ErrorKind::UnexpectedEof {
                break;
            } else {
                println!("{err}");
                return;
            }
        }
    }

    println!("Terminated connection");
}

fn read_char(mut read: impl Read) -> std::io::Result<char> {
    let mut value = [0];
    read.read_exact(&mut value)?;
    let value = value[0];
    char::from_u32(value as u32)
        .filter(char::is_ascii)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Received non-ascii char"))
}

fn process_event(mut read: impl Read, enigo: &mut Enigo) -> std::io::Result<()> {
    let mut value = [0, 0];
    read.read_exact(&mut value)?;
    let [state, mode] = value;

    if state != STATE_PRESSED && state != STATE_RELEASED {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "State is not pressed or released",
        ));
    }

    use enigo::Key::*;

    let key = match mode {
        MODE_CHAR => Layout(read_char(read)?),
        MODE_CTRL => Control,
        MODE_SHIFT => Shift,
        MODE_ALT => Alt,
        MODE_BACKSPACE => Backspace,
        MODE_CAPS_LOCK => CapsLock,
        MODE_DELETE => Delete,
        MODE_DOWN => DownArrow,
        MODE_UP => UpArrow,
        MODE_LEFT => LeftArrow,
        MODE_RIGHT => RightArrow,
        MODE_END => End,
        MODE_ESCAPE => Escape,
        MODE_F1 => F1,
        MODE_F2 => F2,
        MODE_F3 => F3,
        MODE_F4 => F4,
        MODE_F5 => F5,
        MODE_F6 => F6,
        MODE_F7 => F7,
        MODE_F8 => F8,
        MODE_F9 => F9,
        MODE_F10 => F10,
        MODE_F11 => F11,
        MODE_F12 => F12,
        MODE_F13 => F13,
        MODE_F14 => F14,
        MODE_F15 => F15,
        MODE_F16 => F16,
        MODE_F17 => F17,
        MODE_F18 => F18,
        MODE_F19 => F19,
        MODE_F20 => F20,
        MODE_HELP => Help,
        MODE_HOME => Home,
        MODE_META => Meta,
        MODE_OPTION => Option,
        MODE_PAGE_DOWN => PageDown,
        MODE_PAGE_UP => PageUp,
        MODE_RETURN => Return,
        MODE_SPACE => Space,
        MODE_TAB => Tab,
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid mode",
            ))
        }
    };

    if state == STATE_PRESSED {
        enigo.key_down(key);
    } else {
        enigo.key_up(key);
    }

    Ok(())
}

fn connect(ip: IpAddr, port: u16) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("RemoteKeyboard")
        .build(&event_loop)
        .unwrap();

    println!("Connecting to {ip}:{port}");

    let mut stream = match TcpStream::connect((ip, port)) {
        Ok(stream) => stream,
        Err(err) => {
            println!("{err}");
            return;
        }
    };

    println!("Connected to {ip}:{port}");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(code),
                                state,
                                ..
                            },
                        ..
                    },
                window_id,
            } if window_id == window.id() => {
                if let Err(err) = write_event(&mut stream, code, state) {
                    println!("{err}");
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}

fn write_char(mut write: impl Write, value: char) -> std::io::Result<()> {
    if !value.is_ascii() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Cannot send non-ascii character",
        ));
    }
    write.write_all(&[value as u8])
}

fn write_event(
    mut write: impl Write,
    code: VirtualKeyCode,
    state: ElementState,
) -> std::io::Result<()> {
    let (mode, char) = match code {
        VirtualKeyCode::Key1 => (MODE_CHAR, '1'),
        VirtualKeyCode::Key2 => (MODE_CHAR, '2'),
        VirtualKeyCode::Key3 => (MODE_CHAR, '3'),
        VirtualKeyCode::Key4 => (MODE_CHAR, '4'),
        VirtualKeyCode::Key5 => (MODE_CHAR, '5'),
        VirtualKeyCode::Key6 => (MODE_CHAR, '6'),
        VirtualKeyCode::Key7 => (MODE_CHAR, '7'),
        VirtualKeyCode::Key8 => (MODE_CHAR, '8'),
        VirtualKeyCode::Key9 => (MODE_CHAR, '9'),
        VirtualKeyCode::Key0 => (MODE_CHAR, '0'),
        VirtualKeyCode::A => (MODE_CHAR, 'A'),
        VirtualKeyCode::B => (MODE_CHAR, 'B'),
        VirtualKeyCode::C => (MODE_CHAR, 'C'),
        VirtualKeyCode::D => (MODE_CHAR, 'D'),
        VirtualKeyCode::E => (MODE_CHAR, 'E'),
        VirtualKeyCode::F => (MODE_CHAR, 'F'),
        VirtualKeyCode::G => (MODE_CHAR, 'G'),
        VirtualKeyCode::H => (MODE_CHAR, 'H'),
        VirtualKeyCode::I => (MODE_CHAR, 'I'),
        VirtualKeyCode::J => (MODE_CHAR, 'J'),
        VirtualKeyCode::K => (MODE_CHAR, 'K'),
        VirtualKeyCode::L => (MODE_CHAR, 'L'),
        VirtualKeyCode::M => (MODE_CHAR, 'M'),
        VirtualKeyCode::N => (MODE_CHAR, 'N'),
        VirtualKeyCode::O => (MODE_CHAR, 'O'),
        VirtualKeyCode::P => (MODE_CHAR, 'P'),
        VirtualKeyCode::Q => (MODE_CHAR, 'Q'),
        VirtualKeyCode::R => (MODE_CHAR, 'R'),
        VirtualKeyCode::S => (MODE_CHAR, 'S'),
        VirtualKeyCode::T => (MODE_CHAR, 'T'),
        VirtualKeyCode::U => (MODE_CHAR, 'U'),
        VirtualKeyCode::V => (MODE_CHAR, 'V'),
        VirtualKeyCode::W => (MODE_CHAR, 'W'),
        VirtualKeyCode::X => (MODE_CHAR, 'X'),
        VirtualKeyCode::Y => (MODE_CHAR, 'Y'),
        VirtualKeyCode::Z => (MODE_CHAR, 'Z'),
        VirtualKeyCode::Escape => (MODE_ESCAPE, '\0'),
        VirtualKeyCode::F1 => (MODE_F1, '\0'),
        VirtualKeyCode::F2 => (MODE_F2, '\0'),
        VirtualKeyCode::F3 => (MODE_F3, '\0'),
        VirtualKeyCode::F4 => (MODE_F4, '\0'),
        VirtualKeyCode::F5 => (MODE_F5, '\0'),
        VirtualKeyCode::F6 => (MODE_F6, '\0'),
        VirtualKeyCode::F7 => (MODE_F7, '\0'),
        VirtualKeyCode::F8 => (MODE_F8, '\0'),
        VirtualKeyCode::F9 => (MODE_F9, '\0'),
        VirtualKeyCode::F10 => (MODE_F10, '\0'),
        VirtualKeyCode::F11 => (MODE_F11, '\0'),
        VirtualKeyCode::F12 => (MODE_F12, '\0'),
        VirtualKeyCode::F13 => (MODE_F13, '\0'),
        VirtualKeyCode::F14 => (MODE_F14, '\0'),
        VirtualKeyCode::F15 => (MODE_F15, '\0'),
        VirtualKeyCode::F16 => (MODE_F16, '\0'),
        VirtualKeyCode::F17 => (MODE_F17, '\0'),
        VirtualKeyCode::F18 => (MODE_F18, '\0'),
        VirtualKeyCode::F19 => (MODE_F19, '\0'),
        VirtualKeyCode::F20 => (MODE_F20, '\0'),
        VirtualKeyCode::Home => (MODE_HOME, '\0'),
        VirtualKeyCode::Delete => (MODE_DELETE, '\0'),
        VirtualKeyCode::End => (MODE_END, '\0'),
        VirtualKeyCode::PageDown => (MODE_PAGE_DOWN, '\0'),
        VirtualKeyCode::PageUp => (MODE_PAGE_UP, '\0'),
        VirtualKeyCode::Left => (MODE_LEFT, '\0'),
        VirtualKeyCode::Up => (MODE_UP, '\0'),
        VirtualKeyCode::Right => (MODE_RIGHT, '\0'),
        VirtualKeyCode::Down => (MODE_DOWN, '\0'),
        VirtualKeyCode::Back => (MODE_BACKSPACE, '\0'),
        VirtualKeyCode::Return => (MODE_RETURN, '\0'),
        VirtualKeyCode::Space => (MODE_SPACE, '\0'),
        VirtualKeyCode::Numpad0 => (MODE_CHAR, '0'),
        VirtualKeyCode::Numpad1 => (MODE_CHAR, '1'),
        VirtualKeyCode::Numpad2 => (MODE_CHAR, '2'),
        VirtualKeyCode::Numpad3 => (MODE_CHAR, '3'),
        VirtualKeyCode::Numpad4 => (MODE_CHAR, '4'),
        VirtualKeyCode::Numpad5 => (MODE_CHAR, '5'),
        VirtualKeyCode::Numpad6 => (MODE_CHAR, '6'),
        VirtualKeyCode::Numpad7 => (MODE_CHAR, '7'),
        VirtualKeyCode::Numpad8 => (MODE_CHAR, '8'),
        VirtualKeyCode::Numpad9 => (MODE_CHAR, '9'),
        VirtualKeyCode::NumpadAdd => (MODE_CHAR, '+'),
        VirtualKeyCode::NumpadDivide => (MODE_CHAR, '/'),
        VirtualKeyCode::NumpadDecimal => (MODE_CHAR, '.'),
        VirtualKeyCode::NumpadComma => (MODE_CHAR, ','),
        VirtualKeyCode::NumpadEnter => (MODE_RETURN, '\0'),
        VirtualKeyCode::NumpadEquals => (MODE_CHAR, '='),
        VirtualKeyCode::NumpadMultiply => (MODE_CHAR, '*'),
        VirtualKeyCode::NumpadSubtract => (MODE_CHAR, '-'),
        VirtualKeyCode::Apostrophe => (MODE_CHAR, '\''),
        VirtualKeyCode::Asterisk => (MODE_CHAR, '*'),
        VirtualKeyCode::At => (MODE_CHAR, '@'),
        VirtualKeyCode::Backslash => (MODE_CHAR, '\\'),
        VirtualKeyCode::Capital => (MODE_CAPS_LOCK, '\0'),
        VirtualKeyCode::Colon => (MODE_CHAR, ':'),
        VirtualKeyCode::Comma => (MODE_CHAR, ','),
        VirtualKeyCode::Equals => (MODE_CHAR, '='),
        VirtualKeyCode::Grave => (MODE_CHAR, '`'),
        VirtualKeyCode::LAlt => (MODE_ALT, '\0'),
        VirtualKeyCode::LBracket => (MODE_CHAR, '['),
        VirtualKeyCode::LControl => (MODE_CTRL, '\0'),
        VirtualKeyCode::LShift => (MODE_SHIFT, '\0'),
        VirtualKeyCode::LWin => (MODE_META, '\0'),
        VirtualKeyCode::Minus => (MODE_CHAR, '-'),
        VirtualKeyCode::Period => (MODE_CHAR, '.'),
        VirtualKeyCode::Plus => (MODE_CHAR, '+'),
        VirtualKeyCode::RAlt => (MODE_ALT, '\0'),
        VirtualKeyCode::RBracket => (MODE_CHAR, ']'),
        VirtualKeyCode::RControl => (MODE_CTRL, '\0'),
        VirtualKeyCode::RShift => (MODE_SHIFT, '\0'),
        VirtualKeyCode::RWin => (MODE_META, '\0'),
        VirtualKeyCode::Semicolon => (MODE_CHAR, ';'),
        VirtualKeyCode::Slash => (MODE_CHAR, '/'),
        VirtualKeyCode::Tab => (MODE_TAB, '\0'),
        _ => return Ok(()),
    };

    let state = match state {
        ElementState::Pressed => STATE_PRESSED,
        ElementState::Released => STATE_RELEASED,
    };
    write.write_all(&[state, mode])?;
    if mode == MODE_CHAR {
        write_char(&mut write, char)?;
    }
    write.flush()
}
