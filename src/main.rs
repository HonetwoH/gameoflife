use crossterm::{
    cursor, execute,
    style::{self, Stylize},
    terminal::{size, EnterAlternateScreen, LeaveAlternateScreen, SetSize},
    QueueableCommand,
};
use nanorand::{Rng, WyRand};
use std::env;
use std::io::{stdout, Write};
use std::ops::Div;
use std::{thread, time};

const YMAX: usize = 34;
const XMAX: usize = 150;

fn main() {
    let argument: Vec<String> = env::args().skip(1).collect();
    if argument.len() != 3 || argument[0] == "help" {
        eprintln!("Arguments required are: \n<cells> (800-2200) \t<delay> (80-750 millis) \t<generations> (minimum 10)");
    } else {
        // argument parsing
        let alive_cells = argument[0]
            .parse::<usize>()
            .expect("parsing of cells failed");
        let tdelay = argument[1].parse::<u64>().expect("parsing of dealy failed");
        let maxgens = argument[2].parse::<usize>().expect("parsing gen failed");
        let delay = time::Duration::from_secs(1 / 60);
        let (cols, rows) = size().expect("Failed at getting size");

        // create first gen of the cells
        let mut cells = [[(true, false); XMAX]; YMAX];
        first_gen(&mut cells, alive_cells);

        // set terminal
        execute!(stdout(), EnterAlternateScreen).expect("Cannot spawn new window");
        execute!(stdout(), cursor::Hide).expect("Failed to hide cursor");
        execute!(stdout(), SetSize(XMAX as u16, YMAX as u16)).expect("Cannot resize window");

        // loop
        let mut gens = 1;
        while gens <= maxgens {
            cells = next_generation(cells);
            thread::sleep(delay);
            display(&cells, gens);
            gens += 1;
        }

        //reset terminal
        execute!(stdout(), SetSize(cols, rows)).expect("Cannot resize window");
        execute!(stdout(), cursor::Show).expect("Failed to show cursor");
        execute!(stdout(), LeaveAlternateScreen).expect("Cannot exit window");
    }
}

fn display(cells: &[[(bool, bool); XMAX]; YMAX], gen: usize) {
    let mut stdout = stdout();

    stdout
        .queue(cursor::MoveTo(3, 0))
        .expect("paniced which setting gen")
        .queue(style::Print(format!("{:03}", gen)))
        .expect("paniced which setting gen");

    for y in 1..YMAX - 1 {
        for x in 1..XMAX - 1 {
            if cells[y][x].0 {
                if cells[y][x].1 {
                    stdout
                        .queue(cursor::MoveTo(x as u16, y as u16))
                        .expect("cursor jammed !")
                        .queue(style::PrintStyledContent("O".magenta()))
                        .expect("problem in drawing");
                } else {
                    stdout
                        .queue(cursor::MoveTo(x as u16, y as u16))
                        .expect("cursor jammed !")
                        .queue(style::PrintStyledContent(" ".white()))
                        .expect("problem in drawing");
                }
            }
        }
    }
    stdout.flush().expect("flushing failed");
}

fn first_gen(cells: &mut [[(bool, bool); XMAX]; YMAX], alive_cells: usize) {
    let mut rnd = WyRand::new();
    let one_fourth = |x: usize| x.div(4);
    let three_forth = |x| 3 * one_fourth(x);
    for _ in 0..alive_cells {
        let (x, y) = (
            rnd.generate_range(one_fourth(XMAX)..three_forth(XMAX)),
            rnd.generate_range(one_fourth(YMAX)..three_forth(YMAX)),
        );
        cells[y][x].1 = true;
    }
}

fn next_generation(cells: [[(bool, bool); XMAX]; YMAX]) -> [[(bool, bool); XMAX]; YMAX] {
    let mut new_cells = [[(false, false); XMAX]; YMAX];
    for y in 0..YMAX {
        for x in 0..XMAX {
            let alive_neighbors = alive_neighbors(&cells, (y, x));
            new_cells[y][x] = if cells[y][x].1 {
                match alive_neighbors {
                    0..=1 => (true, false), // dead
                    2..=3 => (true, true),  // alive
                    4..=8 => (true, false), // overpopulation
                    _ => panic!("phantom neighbors"),
                }
            } else {
                match alive_neighbors {
                    3 => (true, true),
                    0..=2 => (true, false),
                    4..=8 => (true, false),
                    _ => panic!("phantom neighbors"),
                }
            }
        }
    }
    new_cells
}

fn alive_neighbors(cells: &[[(bool, bool); XMAX]; YMAX], coordinate: (usize, usize)) -> u8 {
    let adjacent: [(i16, i16); 8] = [
        (0, 1),
        (0, -1),
        (1, 0),
        (-1, 0),
        (-1, 1),
        (1, -1),
        (1, 1),
        (-1, -1),
    ];

    let (y, x) = coordinate;
    let alive_neighbors = adjacent
        .map(|(y0, x0)| (y0 + y as i16, x0 + x as i16))
        .into_iter()
        .filter(|(y, x)| (*y as usize) < YMAX && (*x as usize) < XMAX)
        .filter(|(y, x)| cells[*y as usize][*x as usize].1)
        .count();

    alive_neighbors as u8
}
