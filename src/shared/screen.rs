//! This module contains all the logic for switching between alternate screen and main screen.
//!
//! *Nix style applications often utilize an alternate screen buffer, so that they can modify the entire contents of the buffer, without affecting the application that started them.
//! The alternate buffer is exactly the dimensions of the window, without any scrollback region.
//! For an example of this behavior, consider when vim is launched from bash.
//! Vim uses the entirety of the screen to edit the file, then returning to bash leaves the original buffer unchanged.
//!
//! !! Note: this module is only working for unix and windows 10 terminals only.  If you are using windows 10 or lower do not implement this functionality. Work in progress...
//!

use shared::functions;
use state::commands::*;
use Context;

use std::io::{self, Write};
use std::rc::Rc;
use std::convert::From;

pub struct AlternateScreen {
    context: Rc<Context>,
    command_id: u16,
}

impl AlternateScreen {
    /// Get the alternate screen from the context.
    /// By calling this method the current screen will be changed to the alternate screen.
    /// And you get back an handle for that screen.
    pub fn from(context: Rc<Context>) -> Self {
        let command_id = get_to_alternate_screen_command(context.clone());

        let screen = AlternateScreen {
            context: context,
            command_id: command_id,
        };
        screen.to_alternate();
        return screen;
    }

    /// Change the current screen to the mainscreen.
    pub fn to_main(&self) {
        let mut mutex = &self.context.state_manager;
        {
            let mut state_manager = mutex.lock().unwrap();

            let mut mx = &state_manager.get(self.command_id);
            {
                let mut command = mx.lock().unwrap();
                command.undo();
            }
        }
    }

    /// Change the current screen to alternate screen.
    pub fn to_alternate(&self) {
        let mut mutex = &self.context.state_manager;
        {
            let mut state_manager = mutex.lock().unwrap();

            let mut mx = &state_manager.get(self.command_id);
            {
                let mut command = mx.lock().unwrap();
                command.execute();
            }
        }
    }
}

impl Write for AlternateScreen {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut screen = self.context.screen_manager.lock().unwrap();
        {
            screen.write(buf)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut screen = self.context.screen_manager.lock().unwrap();
        {
            screen.flush()
        }
    }
}

impl Drop for AlternateScreen {
    fn drop(&mut self) {
        use CommandManager;

        CommandManager::undo(self.context.clone(), self.command_id);
    }
}

use super::super::shared::environment::Crossterm;

impl From<Crossterm> for AlternateScreen
{
    fn from(crossterm: Crossterm) -> Self {
        let command_id = get_to_alternate_screen_command(crossterm.context());

        let screen = AlternateScreen {
            context: crossterm.context(),
            command_id: command_id,
        };
        screen.to_alternate();
        return screen;
    }
}

// Get the alternate screen command to enable and disable alternate screen based on the current platform
fn get_to_alternate_screen_command(context: Rc<Context>) -> u16 {
    #[cfg(target_os = "windows")]
    let command_id = functions::get_module::<u16>(
        win_commands::ToAlternateScreenBufferCommand::new(context.clone()),
        shared_commands::ToAlternateScreenBufferCommand::new(context.clone()),
    ).unwrap();

    #[cfg(not(target_os = "windows"))]
    let command_id = shared_commands::ToAlternateScreenBufferCommand::new(context.clone());

    return command_id;
}