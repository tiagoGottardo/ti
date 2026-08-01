use std::{panic, process::Command};
use ti::{app::*, cell::get_theme, key::get_key_pressed, render_buffer::RenderBuffer, *};

fn main() -> anyhow::Result<()> {
    let mut app = App::new()?;

    panic::set_hook(Box::new(|panic_info| {
        print!("{DISABLE_MOUSE}{CLEAR_SCREEN}\x1b[2 q");
        Command::new("stty").arg("sane").status().unwrap();

        println!("🚨 Fuck! Some shit happened.");

        if let Some(location) = panic_info.location() {
            println!(
                "On this file: '{}', line: {}",
                location.file(),
                location.line()
            );
        }

        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            println!("Panic message: {s}");
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            println!("Panic message: {s}");
        }
    }));

    Command::new("stty")
        .args(["raw", "-echo", "min", "0", "time", "1"])
        .status()?;
    print!("{ENABLE_MOUSE}{CLEAR_SCREEN}");

    let theme = get_theme()?;

    let mut front_buffer = RenderBuffer::new();
    let mut back_buffer = RenderBuffer::from(&app);

    app.undo
        .push(app.doc.snapshot(), app.cursor.clone(), app.mode);

    loop {
        let diff = back_buffer.diff(&front_buffer);

        prin!(
            "{}{}",
            RenderBuffer::patch(diff, &theme),
            app.cursor.build(&app.doc, &app.viewport, app.mode)
        );

        front_buffer = back_buffer.to_owned();

        if !app.handle_input(get_key_pressed()?)? {
            break;
        }

        app.viewport.fit(&app.cursor, &app.doc);

        back_buffer = RenderBuffer::from(&app);
    }

    print!("{DISABLE_MOUSE}{CLEAR_SCREEN}");
    Command::new("stty").arg("sane").status()?;

    Ok(())
}
