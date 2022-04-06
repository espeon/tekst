use std::{
    io::{stdout, Write},
    time::{Duration, Instant},
};

use crossterm::{cursor, execute, style::{Print, Color, SetForegroundColor}, terminal, ExecutableCommand};

use structs::Lyrics;

mod parse;
mod structs;

fn main() {
    // max time 
    // TODO: refactor game loop into own function 
    let to_elapse = Duration::from_millis(900000);

    // constant delta time
    const DT: u128 = 1 * 1000000; // as nanoseconds (* 1000000)
    let mut accumulator = 00;
    let mut current_time: Instant = Instant::now();
    let mut time = Duration::from_millis(0);
    // debug vars
    let mut update_count = 0;
    let mut frame_count = 0;
    let timer = Instant::now();

    // get lyrics
    let lyrics = parse::parse();
    // set default state and terminal look
    let mut state: (Vec<&String>, usize) = (vec![], 0);
    execute!(stdout(), terminal::Clear(terminal::ClearType::All), cursor::Hide).unwrap();

    // loop
    loop {
        let now = Instant::now();
        // get last iteration's loop time
        let mut frame_time = now - current_time;
        // just in case "tm" our logic can't catch up with our frames
        if frame_time.as_secs_f32() > 0.25 {
            frame_time = Duration::from_millis(0250);
        }

        current_time = now;

        // set accumulator
        accumulator += frame_time.as_nanos();

        // if we can do a tick do it !!!
        // logic loop
        while accumulator >= DT {
            update_count += 1;
            time += Duration::from_nanos(DT.try_into().unwrap());
            state = update(&lyrics, time);
            accumulator -= DT;
        }

        frame_count += 1;

        render(&state);

        // exit
        if timer.elapsed() > to_elapse {
            break;
        }
    }

    println!(
        "took {} s (current time: {} | update count: {} | frame count: {} )",
        timer.elapsed().as_secs_f32(),
        time.as_secs_f32(),
        update_count,
        frame_count
    );
}

fn update(ly: &Lyrics, time: Duration) -> (Vec<&String>, usize) {
    let mut v = vec![];

    let mut i = 0;
    let local_index;

    let lines_count = 16;

    // get current + 1 spot (first line whose timestamp is greater than current)
    for line in &ly.lines {
        if line.start < time {
            i += 1;
        } else {
            break;
        };
    }
    //dbg!((i, time, &ly.lines[i - 1].line));

    // check if we can display the first line
    // if i is 0, then we don't push (it will error) and set
    // our "current value" we send to the render to the first value
    if i > 1 {
        v.push(&ly.lines[i - 2].line);
        local_index = 1;
    } else {
        local_index = 0;
    }

    // probably current active line
    if i > 0 {
        v.push(&ly.lines[i - 1].line);
    }
    // next lines
    for j in 0..lines_count - 2 {
        if i + j < ly.lines.len() {
            v.push(&ly.lines[i + j].line);
        }
    }

    (v, local_index)
}

fn render(lines: &(Vec<&String>, usize)) {
    // y offset
    let mut i = 3;
    for line in &lines.0 {
        // add current marker to current line
        let tp = match lines.1 == i - 3 {
            true => ("> ".to_string() + line),
            _ => line.to_string(),
        };
        // set colours
        let color = match lines.1 == i - 3 {
            true => Color::AnsiValue(15),
            _ => Color::AnsiValue((247 - ((i as f32 / lines.0.len() as f32) * 20.0).log(1.6).ceil() as usize).try_into().unwrap()),
        };
        // idfk what this does
        execute!(
            stdout(),
            cursor::MoveTo(1, i.try_into().unwrap()),
            terminal::Clear(terminal::ClearType::CurrentLine),
            SetForegroundColor(color),
            Print(tp)
        )
        .unwrap();
        i += 1;
    }
}
