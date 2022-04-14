use std::{
    io::stdout,
    time::{Duration, Instant},
};

use clients::Client;
use crossterm::{
    cursor, execute,
    style::{Color, Print, SetForegroundColor},
    terminal,
};

use structs::Lyrics;

use crate::sources::LyricsSource;

mod clients;
mod parse;
mod sources;
mod state;
mod structs;

fn main() {
    loop {
        let client = clients::spotify::SpotifyClient::init();

        let lyrics = sources::xmlyr::XmLyrSource::get(client.get_metadata().unwrap());

        //dbg!(client.get_pos().unwrap());

        setup(lyrics, client);
    }
}

fn setup(lyrics: Lyrics, client: impl Client + Clone) {
    // max time
    // TODO: refactor game loop into own function
    let to_elapse = Duration::from_millis(900000);

    // constant delta time
    const DT: u128 = 1 * 1000000; // as nanoseconds (* 1000000)
    let mut accumulator = 00;
    let mut current_time: Instant = Instant::now();
    let mut time = client.get_pos().unwrap();
    dbg!(&time);

    // debug vars
    let mut update_count = 0;
    let mut frame_count = 0;
    let timer = Instant::now();

    // set default state and terminal look
    let mut state: (Vec<(&String, &Duration)>, usize) = (vec![], 0);
    let time_until_update = 5000; // ms
    let mut next_update_time = 0;
    let mut to_break = false;

    execute!(
        stdout(),
        terminal::Clear(terminal::ClearType::All),
        cursor::Hide
    )
    .unwrap();

    // artist/song metadata

    let title = match &lyrics.metadata.title {
        Some(e) => &e,
        None => "",
    };

    let artist = match (&lyrics.metadata.artist, title) {
        (Some(i), "") => "",
        (Some(i), _) => &i,
        (_, _) => "",
    };

    let separator = match (title, artist) {
        ("", "") => "",
        (_, _) => "-",
    };

    execute!(
        stdout(),
        cursor::MoveTo(1, 1),
        terminal::Clear(terminal::ClearType::CurrentLine),
        SetForegroundColor(Color::AnsiValue(250)),
        Print(format!("{} {} {}", title, separator, artist))
    )
    .unwrap();

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
            if time.as_millis() > next_update_time {
                // get position and metadata for comparing
                // we set position and check metadata against current
                // for song detection
                let pos = client.get_pos().unwrap();

                time = pos;

                let meta = client.get_metadata().unwrap();

                if meta.title.clone().unwrap() != lyrics.metadata.title.to_owned().unwrap() {
                    to_break = true;
                }

                next_update_time += time_until_update;
            }
            state = update(&lyrics, time);
            accumulator -= DT;

            execute!(
                stdout(),
                cursor::MoveTo(1, 0),
                terminal::Clear(terminal::ClearType::CurrentLine),
                Print(time.as_secs_f64())
            )
            .unwrap();
        }

        frame_count += 1;

        render(&state);

        // exit
        if timer.elapsed() > to_elapse || to_break {
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

fn update(ly: &Lyrics, time: Duration) -> (Vec<(&String, &Duration)>, usize) {
    let mut v: Vec<(&String, &Duration)> = vec![];

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
        v.push((&ly.lines[i - 2].line, &ly.lines[i - 2].start));
        local_index = 1;
    } else {
        local_index = 0;
    }

    // probably current active line
    if i > 0 {
        v.push((&ly.lines[i - 1].line, &ly.lines[i - 1].start));
    }
    // next lines
    for j in 0..lines_count - 2 {
        if i + j < ly.lines.len() {
            v.push((&ly.lines[i + j].line, &ly.lines[i + j].start));
        }
    }

    (v, local_index)
}

fn render(lines: &(Vec<(&String, &Duration)>, usize)) {
    // y offset
    let mut i = 3;
    for line in &lines.0 {
        // add current marker to current line
        let tp = match lines.1 == i - 3 {
            true => ("> ".to_string() + line.0),
            _ => line.0.to_string(),
        };
        // set colours
        let color = match lines.1 == i - 3 {
            true => Color::AnsiValue(15),
            _ => Color::AnsiValue(
                (255 - ((std::cmp::max(i, 5) as f32 / lines.0.len() as f32) * 9.0)
                    .log(1.15)
                    .ceil() as usize)
                    .try_into()
                    .unwrap(),
            ),
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
