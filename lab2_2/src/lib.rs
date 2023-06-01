extern crate core;

use std::borrow::Borrow;
use std::env::current_exe;
use std::fmt::Error;
use std::fs;
use std::io::Read;
use std::os::unix::fs::FileExt;
use std::os::unix::prelude::MetadataExt;
use std::time::{SystemTime, UNIX_EPOCH};
use std::vec::IntoIter;

#[derive(Default, Debug)]
pub struct MatchResult<'a, 'b> {
    pub queries: Vec<&'a str>, // query matchate
    pub nodes: Vec<&'b mut File>
}

impl MatchResult<'_, '_ >{

    /*
    pub fn merge(&mut self, item : MatchResult) -> Result<(), Error> {

        for query in &(item.queries) {
            if !self.queries.contains(&query ){
                self.queries.push(query);
            }
        }


        for file in item.nodes{
            if !self.nodes.contains(&file){
                self.nodes.push( file );
            }
        }

        Ok(())
    }
     */

    pub fn check<'a>(queries: &'a [&str], node: &&'a mut File) -> Option<Vec<&'a &'a str>> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        let mut result = vec![];

        for query in queries {
            let expression : Vec<&str> = query.split(":").collect();

            if expression[0].eq("name"){
                if expression[1].eq(&node.name){
                    result.push(query );
                }
            }
            else if expression[0].eq("content"){
                if node.type_ == FileType::Text
                {
                    if String::from_utf8(node.content.clone()).unwrap().contains(&expression[1].to_string()){
                        result.push(query);
                    }
                }
            }
            else if expression[0].eq("larger"){
                if node.content.len() >= expression[1].parse().unwrap(){
                    result.push(query);
                }
            }
            else if expression[0].eq("smaller"){
                if node.content.len() < expression[1].parse().unwrap(){
                    result.push(query);
                }
            }
            else if expression[0].eq("newer"){
                if node.creation_time >= timestamp {
                    result.push(query);
                }
            }
            else if expression[0].eq("older"){
                if node.creation_time < timestamp {
                    result.push(query);
                }
            }
        }

        if result.len() == 0 {
            return None;
        }

        Some(result)
    }

}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum FileType {
    Text, Binary
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct File {
    name: String,
    content: Vec<u8>, // max 1000 bytes, rest of the file truncated
    creation_time: u64,
    type_: FileType,
}

const CONTENT_MAX_SIZE: usize = 1000;

impl File{

    fn bound() -> fn(usize) -> usize {
        |s : usize | {
            if s > CONTENT_MAX_SIZE {
                return CONTENT_MAX_SIZE as usize;
            }
            return s as usize;
        }
    }

    pub  fn new(path : &str) -> Self{
        let metadata = fs::metadata(path).unwrap();

        if ! metadata.is_file() {
            panic!("Error: not a file");
        }

        let size = metadata.size() as usize;

        println!("File size : {} path : {} "  , size , path );

        if size > CONTENT_MAX_SIZE {
            /*
            return  Self{
                name : path.to_string(),
                ..File::default()
            }*/
            println!("The file size overcomes the size limit.")
        }

        //let type_ = match metadata.file_type() { FileType(_) => {} };
        let creation_time = metadata.created().unwrap()
            .duration_since(SystemTime::UNIX_EPOCH).expect("Error time casting")
            .as_millis() as u64;
        let mut content = Vec::<u8>::with_capacity(Self::bound()(size));

        let mut file = fs::File::open(path).unwrap();
        let size = match file.read_to_end(&mut content){
            Ok(size) => { size }
            Err(error) => { panic!("{}",error); }
        };
        assert_eq!(size , content.len());

        let type_ = match path.split('.').last().unwrap(){
            "txt" => {FileType::Text } ,
            _ => { FileType::Binary }
        };

        Self{
            name: path.to_string(),
            content,
            creation_time,
            type_
        }
    }
}

impl Default for File{
    fn default() -> Self {
        Self{
            name: "".to_string(),
            content: vec![],
            creation_time: 0,
            type_: FileType::Text
        }
    }
}

#[derive(Debug, Default)]
pub struct Dir {
    name: String,
    creation_time: u64,
    children: Vec<Node>,
}

impl Dir{
    pub fn new(path: &str) -> Self {

        let metadata = fs::metadata(path).unwrap();

        if ! metadata.is_dir() {
            panic!("Error: the path is not a dir");
        }
        let creation_time = metadata.created().unwrap()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time casting error!")
            .as_millis() as u64;

        let mut children = vec![];
        //Iterator over the dir elements
        for path in  fs::read_dir(path).unwrap() {
            let node = match path {
                Ok(node) => {node}
                Err(err) => { panic!("{}" , err )}
            };
            let node_metadata = node.metadata().unwrap();
            if node_metadata.is_dir() {
                let dir = Dir::new(node.path().to_str().unwrap() );
                children.push(Node::Dir(dir));
            }else if node_metadata.is_file(){
                let file = File::new(node.path().to_str().unwrap());
                children.push(Node::File(file));
            }
        }


        Self{
            name: path.to_string(),
            creation_time,
            children
        }
    }

    pub fn add_child (&mut self , node : Node) -> (){
        self.children.push(node);
    }

    pub fn get_index_by_name(&self , name : &String) -> Option<usize> {
        self.children.iter()
            .position(|node| node.get_name().eq(name))
    }

    pub fn search_r<'a>(&'a mut self , queries : &'a [&'a str]) -> Option<MatchResult> {
        let mut results = MatchResult::default();

        for node in self.children.iter_mut() {
            if node.is_file_mut() {
                let file = node.get_mut_file().unwrap();
                match MatchResult::check(queries , &file ){
                    None => {}
                    Some( q ) => {
                        results.nodes.push(file);
                        for qq in q.into_iter() {
                            if !results.queries.contains(qq){
                                results.queries.push(qq);
                            }
                        }
                    }
                }
            }else if node.is_dir_mut() {
                let dir = node.get_mut_dir().unwrap();
                match dir.search_r(queries) {
                    None => {}
                    Some( result ) => {
                        let nodes = result.nodes;
                        let queries = result.queries;
                        results.nodes.extend(nodes.into_iter());
                        queries.into_iter().for_each(|q| {
                            if !results.queries.contains(&q) {
                                results.queries.push(q);
                            }
                        });
                    }
                }

            }
        }

        if results.queries.len() == 0 {
            return None;
        }

        Some(results)
    }

}

#[derive(Debug)]
pub enum Node {
    File(File),
    Dir(Dir),
}

impl Node{
    pub fn get_name(&self) -> &String{
        match self {
            Node::File(file) => { &file.name }
            Node::Dir(dir) => { &dir.name }
        }
    }

    pub fn is_dir(&self) -> bool {
        match self {
            Node::File(_) => { false }
            Node::Dir(_) => { true }
        }
    }

    pub fn is_file(&self) -> bool {
        match self {
            Node::File(_) => { true }
            Node::Dir(_) => { false }
        }
    }

    pub fn is_dir_mut(&mut self) -> bool {
        match self {
            Node::File(_) => { false }
            Node::Dir(_) => { true }
        }
    }

    pub fn is_file_mut(&mut self) -> bool {
        match self {
            Node::File(_) => { true }
            Node::Dir(_) => { false }
        }
    }


    pub fn get_mut_dir(&mut self ) -> Option<&mut Dir> {
        match self {
            Node::File( _ ) => { None }
            Node::Dir(dir) => { Some( dir) }
            _ => { None }
        }
    }

    pub fn get_dir(& self ) -> Option<& Dir> {
        match self {
            Node::File( _ ) => { None }
            Node::Dir(dir) => { Some( dir) }
            _ => { None }
        }
    }

    pub fn get_mut_file(&mut self ) -> Option<&mut File> {
        match self {
            Node::File( file ) => { Some( file) }
            Node::Dir( _ ) => { None }
            _ => { None }
        }
    }

    pub fn get_file(& self ) -> Option<& File> {
        match self {
            Node::File( file ) => { Some( file) }
            Node::Dir( _ ) => { None }
            _ => { None }
        }
    }


    /*
    pub fn search(&mut self, queries: &[&str]) -> MatchResult{
        let mut results = MatchResult::default();
        let mut curr_dir = self.get_mut_dir().unwrap();

        for node in &mut curr_dir.children {


            if node.is_file(){
                results.merge(MatchResult::check(queries , node.get_mut_file().unwrap()));
            }

            if node.is_dir(){
                let res = node.search(queries);
                results.merge( res );
            }

        }

        results
    }
     */

}

#[derive(Debug, Default)]
pub struct FileSystem {
    root: Dir
}

impl  FileSystem {
    pub fn get_path(path: &str) -> IntoIter<&str> {
        let mut queue: Vec<&str> = path.split("/").collect();
        let mut path = Vec::<String>::new();
        queue.pop();
        while queue.len() > 0 {
            path.push(queue.connect("/"));
            queue.pop();
        }
        queue.into_iter()
    }

    pub fn new() -> Self {
        FileSystem::default()
    }

    pub fn from_dir(path: &str) -> Self {
        Self {
            root: Dir::new(path)
        }
    }

    pub fn mk_dir<'a>(&'a mut self, path_name: &'a str) -> &str {
        let created_dir = match fs::create_dir(&path_name) {
            Ok(()) => { Dir::new(&path_name) }
            Err(error) => { panic!("{}", error); }
        };

        let mut curr_dir: &mut Dir = &mut self.root;
        for step in Self::get_path(&path_name).rev() {
            if step.eq(".") {
                continue;
            }
            if step.ne(&curr_dir.name) {
                match Dir::get_index_by_name(&curr_dir, &step.to_string()) {
                    None => { panic!() }
                    Some(index) => {
                        curr_dir = Node::get_mut_dir(&mut curr_dir.children[index]).unwrap();
                    }
                };
            }
        }
        curr_dir.children.push(Node::Dir(created_dir));
        path_name
    }

    pub fn rm_dir<'a>(&'a mut self, path_name: &'a str) -> &str {
        /*
        Removes an empty directory.
        Errors
        This function will return an error in the following situations, but is not limited to just these cases:
        path doesn't exist.
        path isn't a directory.
        The user lacks permissions to remove the directory at the provided path.
        The directory isn't empty.
        */
        match fs::remove_dir(&path_name) {
            Ok(()) => {}
            Err(error) => { panic!("{}", error); }
        };

        let mut curr_dir: &mut Dir = &mut self.root;
        for step in Self::get_path(&path_name).rev() {
            if step.eq(".") {
                continue;
            }
            if step.ne(&curr_dir.name) {
                match Dir::get_index_by_name(&curr_dir, &step.to_string()) {
                    None => { panic!() }
                    Some(index) => {
                        curr_dir = Node::get_mut_dir(&mut curr_dir.children[index]).unwrap();
                    }
                };
            }
        }

        //Check the child to rm in current_dir
        match Dir::get_index_by_name(&curr_dir, &path_name.to_string()) {
            None => { panic!() }
            Some(index) => {
                curr_dir.children.remove(index)
            }
        };
        path_name
    }

    pub fn new_file<'a>(&'a mut self, path_name: &'a str, file: File) -> &str {
        let mut curr_dir: &mut Dir = &mut self.root;
        for step in Self::get_path(&path_name).rev() {
            if step.eq(".") {
                continue;
            }
            if step.ne(&curr_dir.name) {
                match Dir::get_index_by_name(&curr_dir, &step.to_string()) {
                    None => { panic!() }
                    Some(index) => {
                        curr_dir = Node::get_mut_dir(&mut curr_dir.children[index]).unwrap();
                    }
                };
            }
        }

        curr_dir.children.push(Node::File(file));
        path_name
    }

    pub fn rm_file<'a>(&'a mut self, path_name: &'a str) -> &str {
        fs::remove_file(&path_name);

        let mut curr_dir: &mut Dir = &mut self.root;

        for step in Self::get_path(&path_name).rev() {
            if step.eq(".") {
                continue;
            }
            if step.ne(&curr_dir.name) {
                match Dir::get_index_by_name(&curr_dir, &step.to_string()) {
                    None => { panic!() }
                    Some(index) => {
                        curr_dir = Node::get_mut_dir(&mut curr_dir.children[index]).unwrap();
                    }
                };
            }
        }
        //Check the child to rm in current_dir
        match Dir::get_index_by_name(&curr_dir, &path_name.to_string()) {
            None => { panic!() }
            Some(index) => {
                curr_dir.children.remove(index)
            }
        };
        path_name
    }

    pub fn get_file<'a>(&'a mut self, path_name: &'a str) -> Option<&mut File> {
        let mut curr_dir: &mut Dir = &mut self.root;
        for step in Self::get_path(&path_name).rev() {
            if step.eq(".") {
                continue;
            }
            if step.ne(&curr_dir.name) {
                match Dir::get_index_by_name(&curr_dir, &step.to_string()) {
                    None => { panic!() }
                    Some(index) => {
                        curr_dir = Node::get_mut_dir(&mut curr_dir.children[index]).unwrap();
                    }
                };
            }
        }
        //Check the child to rm in current_dir
        match Dir::get_index_by_name(&curr_dir, &path_name.to_string()) {
            None => { return None; }
            Some(index) => {
                return Node::get_mut_file(&mut curr_dir.children[index]);
            }
        };
    }

    pub fn search<'a >(&'a mut self, queries: &'a [&str]) -> Option<MatchResult> {
        let mut results = MatchResult::default();
        let mut dir_queue = Vec::<&mut Dir>::new();
        let mut dir_queue_index = 0 as usize;
        dir_queue.push(&mut self.root);
        let mut size = &dir_queue.len();

        while dir_queue.len() > 0 {
            let mut dir = dir_queue.pop().unwrap();
            for node in dir.children.iter_mut(){
                if node.is_file_mut(){
                    let file = node.get_mut_file().unwrap();
                    let result =  MatchResult::check(&queries, &file );
                    if result.is_some() {
                        results.nodes.push(file);
                    }
                    for query in result.unwrap() {
                        if !results.queries.contains(&query){
                            results.queries.push(query);
                        }
                    }

                }else if node.is_dir_mut(){
                    let curr_dir = node.get_mut_dir().unwrap();
                    dir_queue.push(curr_dir);
                }
            }
        }

        match results.nodes.len() > 0 {
            true => { Some(results) }
            false => { None }
        }
    }

    //Wrapper function for recursive call
    pub fn search_r<'a >(&'a mut self, queries: &'a [&str]) -> Option<MatchResult>
    {
        let mut root = &mut self.root;
        root.search_r(queries)
    }

}

impl From<Dir> for FileSystem{
    fn from(value: Dir) -> Self {
        Self{
            root: value
        }
    }
}