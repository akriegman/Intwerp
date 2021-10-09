
#![allow(unused)]

use clap::Arg;
use nannou::prelude::*;
use nannou::color::RgbHue;
use std::time::Duration;
use rand::prelude::*;

lazy_static::lazy_static! {
    static ref MATCHES: clap::ArgMatches<'static> = clap::App::new("Intwerp")
        .version("1.0")
        .author("Aaron Kriegman <aaronkplus2@gmail.com>")
        .about("Interpolates mouse movement.")
        .long_about("Interpolates mouse movement. Hold e to draw a path. Press 1, 2, and 3 to switch colors.")
        .arg(
            Arg::with_name("damp")
            .short("d")
            .long("damp")
            .default_value("0.9")
            .help("Decay factor for the momentum of the path.")
        ).arg(
            Arg::with_name("dt")
            .short("t")
            .long("delta")
            .default_value("100")
            .help("Time between arcs, in milliseconds.")
        ).arg(
            Arg::with_name("weight params")
            .short("w")
            .long("weight-params")
            .number_of_values(2)
            .help("Used to set thickness of path in terms of the speed. \
            First number is a multiplier and second number is a threshhold for minimum path speed.")
        ).arg(
            Arg::with_name("clear")
            .short("c")
            .long("clear")
            .help("Resets the screen every frame.")
        ).arg(
            Arg::with_name("auto")
            .short("a")
            .long("auto")
            .help("Turns off auto mode so you have to click to draw each arc. In this mode press e to start a new path.")
        ).get_matches();
    static ref DT: Duration = Duration::from_millis(MATCHES.value_of("dt").unwrap().parse().unwrap());
    static ref DAMP: f32 = MATCHES.value_of("damp").unwrap().parse().unwrap();
    static ref W1: f32 = match MATCHES.values_of("weight params") {
        Some(mut vs) => vs.nth(0).unwrap().parse().unwrap(),
        None => 10.,
    };
    static ref W2: f32 = match MATCHES.values_of("weight params") {
        Some(mut vs) => vs.nth(1).unwrap().parse().unwrap(),
        None => 2.,
    };
    static ref CLEAR: bool = MATCHES.is_present("clear");
    static ref AUTO: bool = !MATCHES.is_present("auto");
    static ref CB: Hsv = Hsv::new(RgbHue::from_radians(random::<f32>() * TAU), 0.2, random::<f32>() / 8.);
    static ref C1: Hsv = Hsv::new(RgbHue::from_radians(random::<f32>() * TAU), 0.5, random::<f32>() / 4. + 0.2);
    static ref C2: Hsv = Hsv::new(RgbHue::from_radians(random::<f32>() * TAU), 0.5, random::<f32>() / 4. + 0.2);
    static ref C3: Hsv = Hsv::new(RgbHue::from_radians(random::<f32>() * TAU), 0.5, random::<f32>() / 4. + 0.2);
}

fn main() {
    *DT; // Force args to be evaluated before creating window
    nannou::app(model).update(update).view(view).run();
}

struct Model {
    path: Vec<Point2>,
    weights: Vec<f32>,
    v_half_last: Point2,
    next_update: Duration,
    first_frame: std::cell::Cell<bool>,
    paused: bool,
    draw_req: bool,
    color: Hsv,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(720, 720)
        .key_pressed(key_pressed)
        .key_released(key_released)
        .mouse_pressed(mouse_pressed)
        .build()
        .unwrap();

    Model {
        path: vec![app.mouse.position()],
        weights: vec![],
        v_half_last: pt2(0., 0.),
        next_update: *DT,
        first_frame: std::cell::Cell::new(true),
        paused: true,
        draw_req: false,
        color: *C1,
    }
}

fn new_line(app: &App, model: &mut Model) {
    model.path = vec![app.mouse.position()];
    model.weights = vec![];
    model.v_half_last = pt2(0., 0.);
}

fn key_pressed(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::E => {
            new_line(app, model);
            model.paused = false;
        }
        Key::Key1 => {
            new_line(app, model);
            model.color = *C1;
        }
        Key::Key2 => {
            new_line(app, model);
            model.color = *C2;
        }
        Key::Key3 => {
            new_line(app, model);
            model.color = *C3;
        }
        _ => (),
    }
}

fn key_released(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::E => {
            model.paused = true;
        }
        _ => (),
    }
}

fn mouse_pressed(app: &App, model: &mut Model, button: MouseButton) {
    match button {
        MouseButton::Left => model.draw_req = true,
        _ => (),
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    if *AUTO {
        if update.since_start < model.next_update {
            return;
        }
        model.next_update += *DT;
        if model.paused {
            return;
        }
    } else {
        if !model.draw_req {
            return;
        }
        model.draw_req = false;
    }

    if !*CLEAR {
        model.path.drain(0..model.path.len() - 1);
        model.weights.clear();
    } else if model.path.len() > 81 {
        model.path.drain(0..model.path.len() - 81);
        model.weights.drain(0..model.weights.len() - 80);
    }

    let p0 = *model.path.last().unwrap();
    let p1 = p0 + model.v_half_last;
    let p2 = app.mouse.position();
    let steps = 20;

    let mut plast = p0;
    for i in 1..=steps {
        let t = (steps - i) as f32 / steps as f32;
        let u = i as f32 / steps as f32;
        let p = t * t * p0 + 2. * t * u * p1 + u * u * p2;
        model.path.push(p);
        model.weights.push(*W1 / (p - plast).length().max(*W2));
        plast = p;
    }

    model.v_half_last = (p2 - p1) * *DAMP;
}

fn view(app: &App, model: &Model, frame: Frame) {
    if model.first_frame.get() || *CLEAR {
        frame.clear(*CB);
        model.first_frame.set(false);
    }
    let draw = app.draw();
    // let win = app.window_rect();

    // let p0 = win.mid_bottom();
    // let p1 = win.xy();
    // let p2 = app.mouse.position();
    // let steps = 50;
    // let mut curve = Vec::<Point2>::with_capacity(steps + 1);
    // for i in 0..=steps {
    //     let t = (steps - i) as f32 / steps as f32;
    //     let u = i           as f32 / steps as f32;
    //     curve.push(t*t * p0 + 2.*t*u * p1 + u*u * p2);
    // }

    // draw.polyline()
    //     .weight(1.)
    //     .color(WHITE)
    //     .points(model.path.clone());

    for i in 0..model.weights.len() {
        draw.line()
            .weight(model.weights[i])
            .color(model.color)
            .points(model.path[i], model.path[i + 1]);
        draw.ellipse()
            .xy(model.path[i])
            .radius(model.weights[i] / 2.)
            .color(model.color);
    }

    draw.to_frame(app, &frame).unwrap();
}
