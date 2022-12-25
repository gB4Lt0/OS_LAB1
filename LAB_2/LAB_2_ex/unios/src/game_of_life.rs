use crate::Screen;
use crate::vga_buf::AsciiChar;

pub const BUF_HEIGHT: u32 = 25;
pub const BUF_WIDTH: u32 = 80;

const MAP: [&str; 25] = [
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                    x                                           ",
    "                                  x x                                           ",
    "                        xx      xx            xx                                ",
    "                       x   x    xx            xx                                ",
    "            xx        x     x   xx                                              ",
    "            xx        x   x xx    x x                                           ",
    "                      x     x       x                                           ",
    "                       x   x                                                    ",
    "                        xx                                                      ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                "
];

pub fn game_of_life(screen: &mut Screen)
{
    let mut current_gen: [[u8; 80]; 25] = [[0; 80]; 25];

    for i in 0..MAP.len()
    {
        for (j, byte) in MAP[i].bytes().enumerate()
        {
            current_gen[i][j] = byte;
        }
    }

    output_to_the_screen(&current_gen,screen);

    loop
    {
        sleep();
        current_gen = next_generation(current_gen);

        output_to_the_screen(&current_gen,screen)
    }

    // TODO: implement game of life
}

pub fn sleep()
{
    let mut n = 4.7632909;
    for i in 0..50000
    {
        n*=n;
    }
}

pub fn output_to_the_screen(area:&[[u8;80];25],screen:&mut Screen)
{
    for i in 0..area.len()
    {
        for j in 0..area[0].len()
        {
            screen.write_byte((i as u32) * BUF_WIDTH + (j as u32), area[i][j]);
        }
    }
}

pub fn finding_all_neighbors_of_a_cell(area:[[u8;80];25], rows:isize, columns:isize) -> u32
{
    let mut count = 0;

    for i in rows - 1 .. rows + 2
    {
        for j in columns - 1 .. columns + 2
        {
            if i == rows && j == columns
            {
                continue;
            }
            if i >= 0 && i < BUF_HEIGHT as isize && j >= 0 && j < BUF_WIDTH as isize
            {
                if area[i as usize][j as usize] == b'x'
                {
                    count += 1;
                }
            }
        }
    }
    return count;
}

pub fn next_generation(area:[[u8; 80]; 25]) -> [[u8; 80]; 25]
{
    let mut next_generation:[[u8;80];25] = [[0;80];25];

    for i in 0 .. area.len()
    {
        for j in 0..area[0].len()
        {
            let mut count_of_neighbors = finding_all_neighbors_of_a_cell(area, i as isize, j as isize);

            if area[i][j] == b'x' && (count_of_neighbors == 3 || count_of_neighbors == 2)
            {
                next_generation[i][j] = b'x';
            }
            else if area[i][j] == b' ' && count_of_neighbors == 3
            {
                next_generation[i][j] = b'x';
            }
            else
            {
                next_generation[i][j] = b' ';
            }
        }

    }
    return next_generation;
}
