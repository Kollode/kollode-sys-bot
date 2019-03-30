#![feature(custom_attribute)]
use futures::Stream;
use tokio_core::reactor::Core;
use telegram_bot::*;
use std::collections::HashMap;

mod config;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let config = match config::Config::from_config() {
        Ok(c) => c,
        Err(e) => panic!("Cant load/parse config file: {}", e)
    };
    let admin_user = telegram_bot::UserId::new(config.admin_user_id);
    let api = Api::configure(config.token.to_owned()).build(&handle).unwrap();

    // Fetch new updates via long poll method
    let future = api.stream().for_each(|update| {

        match update.kind {
            UpdateKind::Message(message) => {
                if message.from.id == admin_user {
                    println!("{:?}", message);
                    handle_message(api.clone(), message, &handle, &config)
                } else {
                    eprintln!("Unknown user tried to call the bot {:?}", message);
                    api.spawn(telegram_bot::SendMessage::new(message.to_source_chat(), "Sorry, I'm not allowed to talk to strangers."))
                }
            },
            UpdateKind::EditedMessage(message) => api.spawn(telegram_bot::SendMessage::new(message.to_source_chat(), "I don't support edited messages.")),
            _ => println!("I don't support this kind of message."),
        }
        Ok(())
    });

    core.run(future).unwrap();
}

fn handle_start(api: Api, message: Message, _handle: &tokio_core::reactor::Handle, commands: &HashMap<String, config::Command>) {
    let mut available_commands: String = String::from("Overview of all configured commands");
    available_commands.push_str(&String::from("\n\n"));
    available_commands.push_str(&String::from("/start -> This view"));

    for (command_string, command) in commands {
        available_commands.push_str(&String::from("\n"));
        available_commands.push_str(&format!("{} -> {}", command_string, command.name));
    }
    api.spawn(telegram_bot::SendMessage::new(message.to_source_chat(), available_commands))
}

fn handle_command(api: Api, message: Message, _handle: &tokio_core::reactor::Handle, command: &config::Command) {
    println!("Command: {:?}", command);
    match std::process::Command::new(&command.script).output() {
        Ok(o) => {
            let output_length = if o.stdout.len() > 4095 { 4095 } else { o.stdout.len()};
            let output = String::from_utf8(o.stdout[0..output_length].to_vec()).expect("Not UTF-8");
            println!("Output: {:?}", output );
            api.spawn(telegram_bot::SendMessage::new(message.to_source_chat(), output))
        },
        Err(e) => {
            println!("Error: {:?}", e);
            api.spawn(telegram_bot::SendMessage::new(message.to_source_chat(), format!("Command {} returned an error: {}", e, e)))
        }
    }
}

fn handle_message(api: Api, message: Message, handle: &tokio_core::reactor::Handle, config: &config::Config) {
    match message.kind {
        MessageKind::Text {ref data, ..} => {
            match data.as_str() {
                "/start" => handle_start(api, message.to_owned(), handle, &config.commands),
                s => if config.commands.contains_key(s) {
                    handle_command(api, message.to_owned(), handle, config.commands.get(s).unwrap().to_owned())
                },
            }
        }
        _ => return
    }
}