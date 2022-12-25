const BUF_ADDR: u32 = 0xb8000;
const BUF_HEIGHT: u32 = 25;
const BUF_WIDTH: u32 = 80;

const COLOR_LIGHT_GREEN: u8 = 0xa;
const COLOR_BLACK: u8 = 0x0;

pub const DEFAULT_COLOR: u8 = (COLOR_BLACK << 4) | COLOR_LIGHT_GREEN;

pub struct AsciiChar 
{
    pub char_byte: u8,
    pub color_byte: u8
}

pub enum Color
{
    White = 0xf,
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Purple = 0x5,
    Brown = 0x6,
    Silver = 0x8,
}

pub enum Alignment 
{
    Left, 
    Right, 
    Center
}

pub struct Screen
{
    buffer: *mut u8,
    color: u8,
    align: Alignment,
    rows: u32,
    columns: u32

}

impl core::fmt::Write for Screen 
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result 
    {
        self.print(s);
        Ok(())
    }
}

impl Screen {
    
    pub fn new (bg_color: Color, text_color: Color, _align: Alignment) -> Screen
    {
        return Screen
        {
            buffer: BUF_ADDR as *mut u8,
            color:((bg_color as u8)<<4) | (text_color as u8),
            columns: Screen ::calculate_align(&_align),
            rows: 0,
            align: _align,


        }
    }

    pub fn calculate_align(align: &Alignment) -> u32
    {
        match align
        {
            Alignment::Left => 0,
            Alignment::Right => BUF_WIDTH,
            Alignment::Center => 0
        }
    }

    pub fn print_hello_world(&mut self) 
    {
        let mut i = 0;
        for byte in "Hello world!".bytes() 
        {
            self.write_char(i, AsciiChar{char_byte: byte, color_byte: self.color});
            i += 1;
        }
    }

    pub fn print(&mut self, s: &str)
    {
        for byte in s.bytes()
        {
            if byte == b'\n'
            {
                if self.rows < BUF_HEIGHT - 1
                {
                    self.rows+=1;
                }
                else
                {
                    self.scroll_up();
                }

                self.columns = Self::calculate_align(&self.align)
            }
            else
            {
                match self.align
                {
                    Alignment::Left =>
                        {
                            self.write_char(self.rows * BUF_WIDTH + self.columns,
                                            AsciiChar{char_byte:byte, color_byte: self.color});
                            self.columns += 1;
                        }
                    Alignment::Right =>
                        {
                            self.move_left();
                            self.write_char(self.rows * BUF_WIDTH + self.columns,
                                            AsciiChar{char_byte: byte, color_byte: self.color})
                        }
                    Alignment::Center =>
                        {
                            if self.columns % 2 !=0
                            {
                                self.move_left();
                            }
                            self.write_char(self.rows * BUF_WIDTH + BUF_WIDTH/2 + self.columns/2,
                                            AsciiChar{char_byte: byte, color_byte:self.color});
                            self.columns+=1;
                        }
                }
            }
        }
    }

    pub fn scroll_up(&self)
    {
        for i in 1..BUF_HEIGHT
        {
            for j in 0 .. BUF_WIDTH
            {
                self.write_char((i-1)*BUF_WIDTH+j,self.read_char(i*BUF_WIDTH+j));
            }
        }

        for i in 0..BUF_WIDTH
        {
            self.write_char((BUF_HEIGHT - 1) * BUF_WIDTH + i,
                            AsciiChar{char_byte: b' ', color_byte: DEFAULT_COLOR})
        }
    }

    pub fn move_left(&self)
    {
        for i in 1 ..BUF_WIDTH
        {
            self.write_char(self.rows * BUF_WIDTH + i - 1,
                             self.read_char(self.rows * BUF_WIDTH + i))
        }
    }

    pub fn write_char(&self, offset: u32, char: AsciiChar) 
    {
        unsafe 
        {
            *self.buffer.offset(offset as isize * 2) = char.char_byte;
            *self.buffer.offset(offset as isize * 2 + 1) = char.color_byte;
        }
    }

    pub fn write_byte(&self, offset: u32, symbol: u8)
    {
        unsafe
        {
            *self.buffer.offset(offset as isize * 2) = symbol;
            *self.buffer.offset(offset as isize * 2 + 1) = self.color;
        }   
    }

    pub fn read_char(&self, offset: u32) -> AsciiChar 
    {
        unsafe 
        {
            return AsciiChar
            {
                char_byte: *self.buffer.offset(offset as isize * 2),
                color_byte: *self.buffer.offset(offset as isize * 2 + 1)
            }
        }
    }
}
