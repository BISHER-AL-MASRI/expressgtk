use adw;
use gtk::{glib, prelude::*, ApplicationWindow, Box, Button, Label};
use std::process::Command;
use std::sync::{Arc, Mutex};

fn main() -> glib::ExitCode {
    // we create the gtk app with libadwitta
    let app = adw::Application::builder()
        .application_id("org.bisheralmasri.ExpressGTK")
        .build();
    // we pass build_ui to the app
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &adw::Application) {
    let connected = Arc::new(Mutex::new(false));
    let button = Button::builder().label("Connect").build();
    let label = Label::builder().label("Disconnected").build();
    // we are creating the div essentially.
    let container = Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .build();
    container.append(&label);
    container.append(&button);

    let connected_clone = Arc::clone(&connected);
    let label_status = label.clone();
    let button_status = button.clone();

    // detecting if the user is already connected
    glib::spawn_future_local(async move {
        let mut connected = connected_clone.lock().unwrap();
        let output = Command::new("expressvpn").arg("status").output();
        if let Ok(output) = output {
            if String::from_utf8_lossy(&output.stdout).contains("Not connected") {
                label_status.set_text("Disconnected");
                button_status.set_label("Connect");
                *connected = false;
            } else {
                label_status.set_text("Connected");
                button_status.set_label("Disconnect");
                *connected = true;
            }
        }
    });

    button.connect_clicked(move |button| {
        let label_clone = label.clone();
        let button_clone = button.clone();
        let connected_clone = Arc::clone(&connected);

        glib::spawn_future_local(async move {
            let mut connected = connected_clone.lock().unwrap();
            let (new_status, new_label, success) = if *connected {
                match Command::new("expressvpn").arg("disconnect").output() {
                    Ok(output) => {
                        if output.status.success() {
                            ("Disconnected", "Connect", true)
                        } else {
                            ("Failed to Disconnect", "Disconnect", false)
                        }
                    }
                    Err(_) => ("Command Error", "Disconnect", false),
                }
            } else {
                match Command::new("expressvpn").arg("connect").output() {
                    Ok(output) => {
                        if output.status.success() {
                            ("Connected", "Disconnect", true)
                        } else {
                            ("Failed to Connect", "Connect", false)
                        }
                    }
                    Err(_) => ("Command Error", "Connect", false),
                }
            };

            if success {
                *connected = !*connected;
            }

            label_clone.set_text(new_status);
            button_clone.set_label(new_label);
        });
    });

    let window = ApplicationWindow::builder()
        .application(app)
        .title("ExpressGTK")
        .child(&container)
        .default_width(200)
        .default_height(150)
        .build();

    // we are setting minimum size of the app to 175, 50
    window.set_size_request(175, 50);
    window.present();
}
