use core::slice::SliceIndex;
use crate::{print, println};
use crate::vga_buf::SCREEN;
use pc_keyboard::DecodedKey;
use lazy_static::lazy_static;

const MAX_SIZE_OF_DIRECTORIES:usize = 20;
const MAX_SIZE_OF_CHILDREN_DIRECTORIES:usize = 10;

const MAX_SIZE_FILES_IN_DIRECTORY:usize = 10;
const MAX_SIZE_FILES:usize = 20;

const CLEAR_MARKER_DIRECTORY:usize = MAX_SIZE_OF_DIRECTORIES + 1;
const CLEAR_MARKER_FILE:usize = MAX_SIZE_FILES + 1;

const MAX_SIZE_DIRECTORY_NAME:usize = 10;

const COMMAND_LENGTH:usize = 10;
const ARGUMENT_LENGTH:usize = 50;

const BUF_HEIGHT:u32 = 25;
const BUF_WIDTH:u32 = 80;
const BUF_SIZE:usize = (BUF_HEIGHT * BUF_WIDTH) as usize;

lazy_static! 
{
    static ref SH: spin::Mutex<Shell> = spin::Mutex::new
    ({
        let mut sh = Shell::new();
        sh
    });
}

pub fn handle_keyboard_interrupt(key: DecodedKey) 
{
    match key 
    {
        DecodedKey::Unicode(c) => SH.lock().on_key_pressed(c as u8),
        DecodedKey::RawKey(rk) => {}
    }
}

pub fn initialize()
{
    print_start();
}

#[derive(Debug, Clone, Copy)]
struct Dir
{
    index:usize,
    name:[u8; MAX_SIZE_DIRECTORY_NAME],
    parent_index:usize,
    child_count:usize,
    child_indexes:[usize; MAX_SIZE_OF_CHILDREN_DIRECTORIES],
    files_indexes:[usize; MAX_SIZE_FILES_IN_DIRECTORY],
}

struct Dirs
{
    dirs:[Dir; MAX_SIZE_OF_DIRECTORIES],
}

#[derive(Debug, Clone, Copy)]
struct File
{
    index:usize,
    name:[u8; MAX_SIZE_DIRECTORY_NAME],
    count_lines:usize,
    folder_index:usize,
    context:[u8;BUF_SIZE],
}

struct Files
{
    files:[File; MAX_SIZE_FILES],
}

pub fn split(array:[u8; 80], buf_len:usize) -> ([u8; COMMAND_LENGTH], [u8; ARGUMENT_LENGTH])
{
    let mut command:[u8; COMMAND_LENGTH] = [b'\0'; COMMAND_LENGTH];
    let mut argument:[u8; ARGUMENT_LENGTH] = [b'\0'; ARGUMENT_LENGTH];

    let mut i = 0;
    while array[i] != b' ' && i < COMMAND_LENGTH
    {
        command[i] = array[i];
        i += 1;
    }
    if i == buf_len - 1
    {
        return (command, argument);
    }

    i += 1;
    let mut j = 0;
    while i < buf_len{
        argument[j] = array[i];
        i += 1;
        j += 1;
    }

    return (command, argument);
}

pub fn compare(line:&str, array:[u8; COMMAND_LENGTH]) -> bool
{
    let mut i = 0;
    for symbol in line.bytes()
    {
        if symbol != array[i]
        {
            return false;
        }
        i += 1;
    }
    return true;
}

fn print_start()
{
    print!(" $ ");
}

fn print_command_not_found(cmd:[u8; COMMAND_LENGTH])
{
    println!();
    print!("[error] Command \'{}\' is not supported", core::str::from_utf8(&cmd).unwrap().trim_matches('\0'));
}

struct Shell 
{
    buf:[u8; 80],
    buf_len:usize,
    dirs:Dirs,
    files:Files,
    curr_dir:usize,
    is_editing_file:bool,
    current_editing_file:usize,
}

impl Shell 
{
    pub fn new() -> Shell 
    {
        let mut shell:Shell = Shell 
        {
            buf: [0; 80],
            buf_len: 0,
            dirs: Dirs{
                dirs: ([Dir
                {
                    index: CLEAR_MARKER_DIRECTORY,
                    name: [b' '; MAX_SIZE_DIRECTORY_NAME],
                    parent_index: 0,
                    child_count: 0,
                    child_indexes: [CLEAR_MARKER_DIRECTORY; MAX_SIZE_OF_CHILDREN_DIRECTORIES],
                    files_indexes: [CLEAR_MARKER_FILE; MAX_SIZE_FILES_IN_DIRECTORY],
                }; MAX_SIZE_OF_DIRECTORIES]),
            },
            curr_dir: 0,
            files: Files
            {
                files: [File
                {
                    index: CLEAR_MARKER_FILE,
                    name: [b'\0'; MAX_SIZE_DIRECTORY_NAME],
                    count_lines: 0,
                    folder_index: CLEAR_MARKER_DIRECTORY,
                    context: [b' '; BUF_SIZE],
                }; MAX_SIZE_FILES]
            },
            is_editing_file: false,
            current_editing_file: CLEAR_MARKER_FILE,
        };

        let root_dir = Dir
        {
            index: 0,
            name: [b'r', b'o', b'o', b't', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0'],
            parent_index: 0,
            child_count: 0,
            child_indexes: [CLEAR_MARKER_DIRECTORY; MAX_SIZE_OF_CHILDREN_DIRECTORIES],
            files_indexes: [CLEAR_MARKER_FILE; MAX_SIZE_FILES_IN_DIRECTORY],
        };

        shell.dirs.dirs[0] = root_dir;
        return shell;
    }

    pub fn on_key_pressed(&mut self, key: u8) 
    {
        match key 
        {
            b'\n' => {
                if self.is_editing_file
                {
                    self.files.files[self.current_editing_file].count_lines += 1;
                    println!();
                    return;
                }

                let argument = split(self.buf, self.buf_len);
                self.command_distributor(argument);
                self.buf_len = 0;

                if self.is_editing_file
                {
                    return;
                }

                println!();
                print_start();
            }
            8 => { // backspace
                if self.is_editing_file
                {
                    SCREEN.lock().delete_last_char(0);
                    return;
                }
                SCREEN.lock().delete_last_char(3);
                if self.buf_len > 0
                {
                    self.buf_len -= 1;
                }
                self.buf[self.buf_len] = 0;
            }
            32 => { // space
                print!("{}", key as char);

                if self.is_editing_file
                {
                    return;
                }

                self.buf[self.buf_len] = b' ';
                self.buf_len += 1;
            }
            96 => { // `
                if self.is_editing_file
                {
                    self.is_editing_file = false;
                    self.files.files[self.current_editing_file].count_lines += 1;
                    self.files.files[self.current_editing_file].context = SCREEN.lock().get_buffer();
                    self.clear();

                    print!("\n[ok] File \"{}\" saved succesfully!\n", core::str::from_utf8(
                        &self.files.files[self.current_editing_file].name.clone()).unwrap().trim_matches('\0'));
                    print_start();
                }
            }
            _ => {
                if self.is_editing_file
                {
                    print!("{}", key as char);
                    return;
                }

                self.buf[self.buf_len] = key;
                self.buf_len += 1;
                print!("{}", key as char);
            }
        }
    }

    fn command_distributor(&mut self, argument:([u8; COMMAND_LENGTH], [u8;ARGUMENT_LENGTH]))
    {
        if compare("cur_dir", argument.0)
        {
            self.cur_dir(self.dirs.dirs[self.curr_dir]);
        }
        else if compare("make_dir", argument.0) 
        {
            self.make_dir(argument.1);
        }
        else if compare("change_dir", argument.0) 
        {
            self.change_dir(argument.1);
        }
        else if compare("dir_tree", argument.0) 
        {
            self.dir_tree(self.dirs.dirs[self.curr_dir]);
        }
        else if compare("remove_dir", argument.0) 
        {
            self.remove_dir(argument.1);
        }
        else if compare("clear", argument.0) 
        {
            self.clear();
        }
        else if compare("make_file", argument.0)
        {
            self.make_file(argument.1);
        }
        else if compare("remove_file", argument.0)
        {
            self.remove_file(argument.1);
        }
        else if compare("dump_file", argument.0)
        {
            self.dump_file(argument.1);
        }
        else if compare("edit_file", argument.0)
        {
            self.edit_file(argument.1)
        }
        else 
        {
            print_command_not_found(argument.0);
        }
    }

    fn cur_dir(&mut self, curr_dir:Dir) -> usize{
        let mut tree = 0;
        if curr_dir.index > 0{
            tree = self.cur_dir(self.dirs.dirs[curr_dir.parent_index]);
        }
        else{
            println!();
        }
        print!("/{}", core::str::from_utf8(&curr_dir.name.clone()).unwrap().trim_matches('\0'));
        return tree;
    }

    fn make_dir(&mut self, argument:[u8; ARGUMENT_LENGTH])
    {
        let mut name_size = 0;
        for i in 0..ARGUMENT_LENGTH
        {
            if argument[i] == b'\0'
            {
                break;
            }
            name_size+=1;
        }

        if  name_size == 0
        {
            print!("\n[Error] Specify a name of folder");
            return;
        }


        if name_size > MAX_SIZE_DIRECTORY_NAME
        {
            print!("\n[Error] The maximum number of characters has been exceeded");
            return;
        }

        if self.get_folder_index_by_name(argument) != CLEAR_MARKER_DIRECTORY
        {
            print!("\n[Error] Directory \"{}\" already exist", core::str::from_utf8(&argument.clone())
                .unwrap()
                .trim_matches('\0'));
            return;
        }

        let mut dir_index = CLEAR_MARKER_DIRECTORY;
        for i in 0..MAX_SIZE_OF_DIRECTORIES{
            if self.dirs.dirs[i].index == CLEAR_MARKER_DIRECTORY{
                dir_index = i;
                break;
            }
        }

        if dir_index == CLEAR_MARKER_DIRECTORY{
            print!("\n[Error] The maximum number of directories");
            return;
        }

        let mut free_index = 0;
        for i in 0..MAX_SIZE_OF_CHILDREN_DIRECTORIES{
            if self.dirs.dirs[self.curr_dir].child_indexes[i] == CLEAR_MARKER_DIRECTORY{
                free_index = i;
                break;
            }
        }

        if free_index == CLEAR_MARKER_DIRECTORY{
            print!("\n[Error] The maximum number of children directories");
            return;
        }

        let mut directory: Dir = Dir{
            index: dir_index,
            name: [b'\0'; MAX_SIZE_DIRECTORY_NAME],
            parent_index: self.curr_dir,
            child_count: 0,
            child_indexes: [CLEAR_MARKER_DIRECTORY; MAX_SIZE_OF_CHILDREN_DIRECTORIES],
            files_indexes: [CLEAR_MARKER_FILE; MAX_SIZE_FILES_IN_DIRECTORY],
        };
        for i in 0..MAX_SIZE_DIRECTORY_NAME{
            directory.name[i] = argument[i];
        }
        self.dirs.dirs[dir_index] = directory;
        self.dirs.dirs[self.curr_dir].child_indexes[free_index] = dir_index;
        self.dirs.dirs[self.curr_dir].child_count += 1;

        print!("\n[ok] Created new dir \'{}\'", core::str::from_utf8(&directory.name.clone()).unwrap().trim_matches('\0'));
    }

    fn change_dir(&mut self, argument:[u8; ARGUMENT_LENGTH])
    {
        if argument[0] == b'.'
        {
            self.curr_dir = self.dirs.dirs[self.curr_dir].parent_index;
            print!("\n[Ok] Directory has changed");
            return;
        }
        let current_dir = self.dirs.dirs[self.curr_dir];

        for dir_index in current_dir.child_indexes{

            let mut is_same = true;

            if dir_index == CLEAR_MARKER_DIRECTORY
            {
                continue;
            }

            for i in 0..ARGUMENT_LENGTH
            {
                if argument[i] == b'\0'
                {
                    break;
                }
                if i == MAX_SIZE_DIRECTORY_NAME{
                    print!("[Error] The maximum number of characters has been exceeded");
                    return;
                }

                if self.dirs.dirs[dir_index].name[i] != argument[i]
                {
                    is_same = false;
                    break;
                }
            }
            if is_same
            {
                self.curr_dir = self.dirs.dirs[dir_index].index;
                print!("\n[Ok] Directory has changed");
                return;
            }
        }
        print!("\nFolder \'{}\' does not exist", core::str::from_utf8(&argument.clone()).unwrap().trim_matches('\0'));
    }

    fn remove_dir(&mut self, dir_name: [u8; ARGUMENT_LENGTH]){
        let mut is_correct = false;

        for i in 0..MAX_SIZE_DIRECTORY_NAME{
            if dir_name[i] != b'\0'
            {
                is_correct = true;
                break;
            }
        }
        if !is_correct{
            print!("\n[Error] Specify a name of folder");
            return;
        }

        let cur_dir = self.dirs.dirs[self.curr_dir];
        for i in 0..cur_dir.child_count{
            let mut is_same = true;
            let checking_dir = self.dirs.dirs[cur_dir.child_indexes[i]];
            for j in 0..MAX_SIZE_DIRECTORY_NAME{
                if checking_dir.name[j] != dir_name[j]{
                    is_same = false;
                    break;
                }
            }
            if !is_same{
                continue;
            }

            if self.dirs.dirs[checking_dir.index].child_count > 0{
                print!("[Error] Cannot delete a directory with children");
                return;
            }
            self.dirs.dirs[self.curr_dir].child_count -= 1;
            self.dirs.dirs[checking_dir.index] = Dir{
                index: CLEAR_MARKER_DIRECTORY,
                name: [b' '; MAX_SIZE_DIRECTORY_NAME],
                parent_index: CLEAR_MARKER_DIRECTORY,
                child_count: CLEAR_MARKER_DIRECTORY,
                child_indexes: [CLEAR_MARKER_DIRECTORY; MAX_SIZE_OF_CHILDREN_DIRECTORIES],
                files_indexes: [CLEAR_MARKER_FILE; MAX_SIZE_FILES_IN_DIRECTORY],
            };
            self.dirs.dirs[cur_dir.index].child_indexes[i] = CLEAR_MARKER_DIRECTORY;
            print!("\n[Ok] Directory \"{}\" deleted", core::str::from_utf8(&dir_name.clone())
                .unwrap().trim_matches('\0'));
            return;
        }
    }

    fn dir_tree(&mut self, current_directory:Dir)
    {
        println!();
        print!("/{}", core::str::from_utf8(&current_directory.name).unwrap().trim_matches('\0'));
        self.print_children_dirs(current_directory, 1);
    }

    fn print_children_dirs(&mut self, current_directory:Dir, tab_count:usize)
    {
        println!();
        for i in 0..MAX_SIZE_OF_CHILDREN_DIRECTORIES
        {
            if current_directory.child_indexes[i] == CLEAR_MARKER_DIRECTORY
            {
                continue;
            }

            let child_directory = self.dirs.dirs[current_directory.child_indexes[i]];
            for tabs in 0..tab_count
            {
                for tabs in 0..4
                {
                    print!(" ");
                }
            }
            print!("/{}", core::str::from_utf8(&child_directory.name).unwrap().trim_matches('\0'));
            self.print_children_dirs(child_directory, tab_count + 1);
        }
        for i in 0..MAX_SIZE_FILES_IN_DIRECTORY
        {
            if current_directory.files_indexes[i] != CLEAR_MARKER_FILE
            {
                for tabs in 0..tab_count
                {
                    for tabs in 0..4{

                        print!(" ");
                    }
                }
                print!("/{}.txt", core::str::from_utf8(&self.files.files[current_directory.files_indexes[i]].name).unwrap().trim_matches('\0'));
                println!();
            }
        }
    }

    fn get_folder_index_by_name(&mut self, argv: [u8; ARGUMENT_LENGTH]) -> usize
    {
        for i in 0..MAX_SIZE_OF_CHILDREN_DIRECTORIES
        {
            let mut is_same = true;
            let dir_index = self.dirs.dirs[self.curr_dir].child_indexes[i];

            if dir_index == CLEAR_MARKER_DIRECTORY
            {
                continue;
            }

            for i in 0..ARGUMENT_LENGTH
            {
                if argv[i] == b'\0'
                {
                    break;
                }

                if self.dirs.dirs[dir_index].name[i] != argv[i]
                {
                    is_same = false;
                    break;
                }
            }

            if is_same {
                // return index of child_indeexes array of current folder
                return i;
            }
        }
        return CLEAR_MARKER_DIRECTORY;
    }


    fn clear(&mut self)
    {
        SCREEN.lock().clear();
    }

    fn make_file(&mut self, argument:[u8; ARGUMENT_LENGTH])
    {
        let mut name_length = 0;
        let mut name = [b'\0'; MAX_SIZE_DIRECTORY_NAME];

        for i in 0..ARGUMENT_LENGTH
        {
            if argument[i] == b'\0'
            {
                break;
            }
            name[i] = argument[i];
            name_length += 1;
        }
        if name_length > MAX_SIZE_DIRECTORY_NAME
        {
            print!("\n[Error] The maximum number of characters has been exceeded");
            return;
        }

        let mut file_index = CLEAR_MARKER_FILE;

        for i in 0..MAX_SIZE_FILES_IN_DIRECTORY
        {
            if self.files.files[i].index == CLEAR_MARKER_FILE
            {
                file_index = i;
                break;
            }
        }
        if file_index == CLEAR_MARKER_FILE
        {
            print!("\n[Error] The maximum number of files in directory");
        }

        for i in 0..MAX_SIZE_FILES
        {
            if self.files.files[i].index == CLEAR_MARKER_FILE
            {
                file_index = i;
                break;
            }
        }
        if file_index == CLEAR_MARKER_FILE{
            print!("\n[Error] The maximum number of files");
        }

        let mut file = File
        {
            index: file_index,
            name: name,
            count_lines: 0,
            folder_index: self.curr_dir,
            context: [b' '; BUF_SIZE],
        };
        self.is_editing_file = true;
        self.current_editing_file = file_index;
        SCREEN.lock().clear();

        self.files.files[file_index] = file;

        let mut index_for_folder = CLEAR_MARKER_FILE;
        for i in 0..MAX_SIZE_OF_CHILDREN_DIRECTORIES{
            if self.dirs.dirs[self.curr_dir].files_indexes[i] == CLEAR_MARKER_FILE{
                index_for_folder = i;
                break;
            }
        }

        self.dirs.dirs[self.curr_dir].files_indexes[index_for_folder] = file_index;
    }

    fn remove_file(&mut self, argument:[u8; ARGUMENT_LENGTH])
    {
        self.clear();
        let cur_file_index = self.get_file_index(argument);
        if cur_file_index == CLEAR_MARKER_FILE{
            print!("\n[Error] File does not exist");
            return;
        }

        self.files.files[cur_file_index] = File
        {
            index: CLEAR_MARKER_FILE,
            name: [b'\0'; MAX_SIZE_DIRECTORY_NAME],
            count_lines: 0,
            folder_index: CLEAR_MARKER_DIRECTORY,
            context: [b' '; BUF_SIZE],
        };

        for i in 0..MAX_SIZE_FILES_IN_DIRECTORY
        {
            if self.dirs.dirs[self.curr_dir].child_indexes[i] == cur_file_index
            {
                self.dirs.dirs[self.curr_dir].child_indexes[i] = CLEAR_MARKER_FILE;
            }
        }
    }

    fn dump_file(&mut self, argument: [u8; ARGUMENT_LENGTH])
    {
        self.clear();
        let cur_file_index = self.get_file_index(argument);
        if cur_file_index == CLEAR_MARKER_FILE
        {
            print!("\n[Error] File \"{}\" does not exist!", core::str::from_utf8(&argument.clone()).unwrap().trim_matches('\0'));
            return;
        }

        for i in 0..(BUF_WIDTH * (self.files.files[cur_file_index].count_lines) as u32)
        {
            print!("{}", self.files.files[cur_file_index].context[i as usize] as char);
        }
    }

    fn edit_file(&mut self, argument:[u8; ARGUMENT_LENGTH])
    {
        let cur_file_index = self.get_file_index(argument);
        if cur_file_index == CLEAR_MARKER_FILE
        {
            print!("\n[Error] File \"{}\" does not exist!", core::str::from_utf8(&argument.clone()).unwrap().trim_matches('\0'));
            return;
        }

        self.is_editing_file = true;
        self.current_editing_file = cur_file_index;
        self.files.files[self.current_editing_file].count_lines = 0;
        self.clear();
    }

    fn get_file_index(&mut self, argument:[u8; ARGUMENT_LENGTH]) -> usize {
        let mut cur_file_index = CLEAR_MARKER_FILE;
        let mut is_same = true;
        for i in 0..MAX_SIZE_FILES_IN_DIRECTORY{
            cur_file_index = self.dirs.dirs[self.curr_dir].files_indexes[i];
            if cur_file_index == CLEAR_MARKER_FILE{
                continue;
            }
            is_same = true;
            for j in 0..MAX_SIZE_DIRECTORY_NAME{
                if argument[j] == b'\0'{
                    break;
                }
                if argument[j] != self.files.files[cur_file_index].name[j]{
                    is_same = false;
                    break;
                }
            }
            if is_same {
                return cur_file_index;
            }
        }
        return CLEAR_MARKER_FILE;
    }
}