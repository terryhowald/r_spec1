#![allow(warnings)]
use eframe::egui;
use egui_plot::{Legend, Line, BarChart, Bar, Plot, PlotPoints};
use egui::{Visuals, Vec2, Vec2b, Color32, Stroke};

use rand::Rng;
use std::sync::{Arc, Mutex};

use rppal::i2c::I2c;

fn slow_process(state_clone: Arc<Mutex<State>>) {
    // Get access to Raspberry Pi's I2C peripheral
    let mut i2c: I2c = I2c::new().unwrap();

    // Inialize AS7341 for I2C communication
    r_spec1::as7341_init(&mut i2c);

    // AS7341 ATIME Config
    r_spec1::as7341_atime_config(&i2c, 100);

    // AS7341 ASTEP Config
    r_spec1::as7341_astep_config(&i2c, 999);

    // AS7341 AGAIN Configs
    r_spec1::as7341_again_config(&i2c, 0x06);

    // Enable AS7341
    r_spec1::as7341_enable(&i2c, true);

    // Enable and turn on LEDs
    //r_spec1::as7341_enable_leds(&i2c, true);
    //r_spec1::as7341_control_leds(&i2c, true, 18);  

    loop {
        // Give up time slice to let GUI repaint get through
        std::thread::sleep(std::time::Duration::from_millis(500));
   
        // Get data from bank 0
        r_spec1::as7341_start_measure(&i2c, 0);
        r_spec1::as7341_read_spectral_data_one(&i2c);
        for n in 0..4 as usize {
            let val: f64 = (r_spec1::as7341_get_channel_data(&i2c, (n as u8)) as f64);
            state_clone.lock().unwrap().bars[n] = val;
        }

        // Get data from bank 1
        r_spec1::as7341_start_measure(&i2c, 1);
        r_spec1::as7341_read_spectral_data_two(&i2c);
        for n in 0..4 as usize {
            let val: f64 = (r_spec1::as7341_get_channel_data(&i2c, (n as u8)) as f64);
            state_clone.lock().unwrap().bars[n+4] = val;
        }        
        
        // Get GUI context and send repaint request
        let ctx = &state_clone.lock().unwrap().ctx;
        match ctx {
            Some(x) => x.request_repaint(),
            None => panic!("error in Option<>"),
        }
    }

    // Turn off and disable LEDs
    //r_spec1::as7341_control_leds(&i2c, false, 0);
    //r_spec1::as7341_enable_leds(&i2c, false); 
}

struct State {
    bars: [f64; 8],
    ctx: Option<egui::Context>,
}

impl State {
    pub fn new() -> Self {
        Self {
            bars: [0.0; 8],
            ctx: None,
        }
    }
}

fn bar_color(index: i32) -> Color32 {
    // Determine color from index
    let mut hex = "#000000";
    match index {
        1 => { // Violet
            hex = "#EE82EE";
        }
        2 => { // Indigo
            hex = "#4B0082";
        }
        3 => { // Blue
            hex = "#0000FF";
        }
        4 => { // Cyan
            hex = "#00FFFF";
        }
        5 => { // Green
            hex = "#008000";
        }
        6 => { // Yellow
            hex = "#FFFF00";
        }
        7 => { // Orange
            hex = "#FFA500";
        }
        8 => { // Red
            hex = "#FF0000";
        }
        _ => { // Default
            hex = "#000000";
        }
    }

    // Return indexed color
    return(Color32::from_hex(hex).unwrap())
}

pub struct App {
    state: Arc<Mutex<State>>, 
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let state = Arc::new(Mutex::new(State::new()));
        state.lock().unwrap().ctx = Some(cc.egui_ctx.clone());
        let state_clone = state.clone();
        std::thread::spawn(move || {
            slow_process(state_clone);
        });
        Self {
            state,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        egui::CentralPanel::default().show(ctx, |ui| {

            let my_plot = Plot::new("My Plot")
                .legend(Legend::default())
                .allow_boxed_zoom(false)
                .allow_drag(false)
                .allow_scroll(false)
                .allow_double_click_reset(false)
                .show_grid(true)
                .x_axis_label("Color".to_string())
                .y_axis_label("Power".to_string())
                .show_x(false)
                .show_y(false)
                .include_y(65536.0);

            // Create fake data for barchart
            let mut bars: Vec<Bar> = Vec::new();
            // Setup bar for each color
            for n in 0..8 {
                let arg = (n+1) as f64 * 1.0;
                let val = self.state.lock().unwrap().bars[n];
                let bar = Bar::new(arg, val)
                    .stroke(Stroke::new(1.0, Color32::BLACK))
                    .fill(bar_color((n+1)as i32))
                    .width(1.0);
                bars.push(bar);
            }

            my_plot.show(ui, |plot_ui| {
                plot_ui.bar_chart(BarChart::new(bars)
                );
            });

        });
    }
}

fn main() {

    let native_options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
    ..Default::default()
    };

    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    ).unwrap();
}
