#![allow(warnings)]

use tokio_postgres::{ Client, NoTls };
use tokio_postgres::Error as PostgresError;
use tokio::{
    io::{BufStream},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use std::sync::Arc;
use std::env;

#[tokio::main]
async fn main()  -> std::io::Result<()> {
    use resp::*;
    use req::*;
    use tokio_postgres::types::BorrowToSql;
    use std::io::Cursor;

    let listener: TcpListener = TcpListener::bind("127.0.0.1:7878").await.expect("Failed to bind tcplistener; used port");
    println!("Now listening on: {}", listener.local_addr().unwrap());

    //Connect to the database
    let (client, connection) =
        tokio_postgres::connect("hostaddr=127.0.0.1 dbname=logindb", NoTls).await.unwrap();
    let client = Arc::new(Mutex::new(client));
    println!("Connection established to database");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    
    loop{
        let (stream, _addr) = match listener.accept().await{
            Ok((s, a)) => (s, a),
            Err(e) => {
                println!("Error connecting: {e}");
                continue
            }
        };
        let mut stream = BufStream::new(stream);
        let my_client = Arc::clone(&client);
        
        // println!("Connection Established to addr: {_addr}");

        tokio::spawn(async move{
            // let mut buf: &[u8] = &[0; 2048];

            // stream.read_line(&mut buf).await?;
            loop{
                let mut request = req::parse_request(&mut stream).await.unwrap();

                let method = request.method;
    
                // match method{
                //     req::Method::GET => (),
                //     req::Method::POST => (),
                //     req::Method::PUT => (),
                //     req::Method::OPTIONS => (),
                //     req::Method::DELETE => (),
                // }
    
                
                
                // resp::Response::err404().write(&mut stream).await.unwrap();
                if (method == Method::GET && request.path == "/".to_string()){
                    send_page("../frontend/out/index.html", ContentType::html, &mut stream)
                        .await
                        .unwrap();
                }else if (method == Method::GET && request.path == "/chat".to_string()){
                    send_page("../frontend/out/chat.html", ContentType::html, &mut stream)
                        .await
                        .unwrap();
                
                }else if (method == Method::GET){
                    match send_page("../frontend/out".to_owned() + &request.path, ContentType::from_path(&request.path).unwrap(), &mut stream)
                        .await{
                            Ok(_) => (),
                            Err(_) => Response::err404().write(&mut stream).await.unwrap(),
                        }
                }else if method == Method::POST && request.headers.get("Authorization").is_some() {
                    let auth = &request.headers.get("Authorization").unwrap()[5..];
                    // println!("Before slice: {}", &request.headers.get("Authorization").unwrap());
                    println!("After slice: {}", &request.headers.get("Authorization").unwrap()[5..]);
                    
                    // println!("{request:?}");
                    // let json = serde_json::from_str(
                    let user: User = serde_json::from_str(&auth.trim()).unwrap();
                    // let query_params = [user.email.borrow_to_sql(), user.password.borrow_to_sql()];
                    // println!("{user:?}");
                    // println!("Accepts str? {}", )
                    let result = my_client.lock().await.query_opt("SELECT id, name FROM users WHERE email = $1 AND password = $2;", &[&user.email.as_str(), &user.password.as_str()]).await.unwrap();
                    match result {
                        Some(r) => {
                            //
                            // Send 200: OK Response with Cookie
                            //
                            println!("User found");
                            let status_code = Status::Ok;

                            let headers = maplit::hashmap!{
                                "Content-Length".to_owned() => "0".to_owned(),
                                "Set-Cookie".to_owned() => format!("signin={}", auth),
                            };

                            let response = Response{
                                status: status_code,
                                headers,
                                payload: Cursor::new(Vec::new())
                            };

                            response.clone().write(&mut stream).await.unwrap();
                            println!("Sent response: {response:?}");
                        },
                        None => {
                            println!("User not found");
                            let response = Response::incorrect_login();
                            
                            response.clone().write(&mut stream).await.unwrap();
                            // println!("Sent response: {response:?}");
                        },
                    }
                    
                    // println!("Query result: {result}");
                } else if method == Method::PUT && request.path.contains("/chats"){
                    let result = my_client.lock().await.query_opt()

                } else if method == Method::PUT && request.path.contains("/users"){

                }
                }else{
                    println!("404 Request: {request:?}");
                    Response::err404().write(&mut stream).await.unwrap();
                }

            }
        });
    }

    Ok(())
}


#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Deserialize, Debug)]
pub struct User{
    id: Option<i32>,
    name: Option<String>,
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Message{
    content: String,
    sent_by: String, //user's display name
}

#[derive(Serialize, Deserialize)]
pub struct ChatRoom{
    messages: Vec<Message>,
}

async fn send_page<S>(
    path: S,
    content_type: crate::resp::ContentType,
    stream: &mut BufStream<TcpStream>,
) -> Result<(), anyhow::Error>
where
    S: ToString,
{
    use std::{
        io::{self, Read},
        fs,
    };
    use crate::resp::*;

    let path = path.to_string();
    let mut file = match fs::File::open(path.clone()) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Could not find file: {} while sending page", path);
            return Err(e.into());
        }
    };
    let mut bytes = Vec::new();
    let len = file.read_to_end(&mut bytes).unwrap();

    let headers = maplit::hashmap! {
        "Content-Type".to_owned() => content_type.to_string(),
        "Content-Length".to_owned() => len.to_string(),
    };

    let resp = Response {
        status: Status::Ok,
        headers,
        payload: io::Cursor::new(bytes),
    };

    resp.write(stream).await?;

    Ok(())
}

pub mod req{
    use anyhow::anyhow;
    use std::collections::HashMap;
    use std::pin::Pin;
    use tokio::{
        io::{BufStream, AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncReadExt},
        net::TcpStream,
    };

    #[derive(Debug, Clone)]
    pub struct Request {
        pub method: Method,
        pub path: String,
        pub headers: HashMap<String, String>,
        pub body: String,
    }

    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
    pub enum Method {
        OPTIONS,
        GET,
        POST,
        PUT,
        DELETE,
    }

    impl TryFrom<&str> for Method {
        type Error = anyhow::Error;

        fn try_from(mut value: &str) -> std::result::Result<Self, Self::Error> {
            value = value.trim();
            match value {
                "GET" => Ok(Method::GET),
                "POST" => Ok(Method::POST),
                "PUT" => Ok(Method::PUT),
                "DELETE" => Ok(Method::DELETE),
                "OPTIONS" => Ok(Method::OPTIONS),
                m => Err(anyhow!("Invalid method: {m:?}")),
            }
        }
    }

    pub async fn parse_request<T>(mut stream: &mut T) -> anyhow::Result<Request> 
    where
        T: AsyncBufRead + Unpin + AsyncRead + AsyncWrite + std::fmt::Debug
    {
        // println!("LINES: {:?}", stream.lines());
        let mut line_buffer = String::with_capacity(8192);
        stream.read_line(&mut line_buffer).await?;
        // println!("{}", line_buffer);
        

        let mut parts = line_buffer.split_whitespace();

        let method: Method = parts
            .next()
            .ok_or({ anyhow::anyhow!("missing method in req: {}", line_buffer) })
            .and_then(TryInto::try_into)?;

        let path: String = parts
            .next()
            .ok_or(anyhow::anyhow!("missing path"))
            .map(Into::into)?;

        let mut headers = HashMap::new();
        
        

        loop {
            line_buffer.clear();
            stream.read_line(&mut line_buffer).await?;

            if line_buffer.is_empty() || line_buffer == "\n" || line_buffer == "\r\n" {
                break;
            }

            let mut comps = line_buffer.splitn(2, ":");
            let key = comps.next().ok_or(anyhow::anyhow!("missing header name"))?;
            let value = comps
                .next()
                .ok_or(anyhow::anyhow!("missing header value"))?
                .trim();

            headers.insert(key.to_string(), value.to_string());
        }

        let mut body = String::new();


        line_buffer.clear();


        if ( method == Method::POST || method == Method::PUT) && i32::from_str_radix(&*headers.get("Content-Length").ok_or("").unwrap(), 10).unwrap() > 0 {
            let body_len = i32::from_str_radix(&*headers.get("Content-Length").unwrap(), 10)?;
            let mut line_buffer = vec![0u8; body_len as usize];
            stream.read_exact(&mut line_buffer).await?;
            body = String::from_utf8(line_buffer)?;
        }

        Ok(Request {
            method,
            path,
            headers,
            body,
        })
    }
}

mod resp {
    use std::{
        collections::HashMap,
        fs::{self},
        hash::Hash,
        io::{self, Bytes, Cursor, Read},
        str::FromStr,
    };

    use anyhow::Ok;
    use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

    #[derive(Debug, Clone)]
    pub struct Response<S: AsyncRead + Unpin> {
        pub status: Status,
        pub headers: HashMap<String, String>,
        pub payload: S,
    }

    impl Response<io::Cursor<Vec<u8>>> {
        pub fn favicon_resp() -> Self {
            let mut favicon = fs::File::open("images/favicon.ico").unwrap();
            let mut bytes = Vec::new();
            let len = favicon.read_to_end(&mut bytes).unwrap();

            let headers = maplit::hashmap! {
                "Content-Type".to_owned() => "image/x-icon".to_owned(),
                "Content-Length".to_owned() => len.to_string(),
            };

            Self {
                status: Status::Ok,
                headers,
                payload: io::Cursor::new(bytes),
            }
        }

        pub fn err404Page() -> Self {
            let mut fBytes = Vec::new();
            let len =
                fs::File::read_to_end(&mut fs::File::open("html/404.html").unwrap(), &mut fBytes)
                    .unwrap();
            let status = Status::NotFound;

            let headers = maplit::hashmap! {
                "Content-Type".to_owned() => "text/html".to_owned(),
                "Content-Length".to_owned() => len.to_string(),
            };

            Self {
                status,
                headers,
                payload: io::Cursor::new(fBytes),
            }
        }

        pub fn err404() -> Self {
            Self {
                status: Status::NotFound,
                headers: HashMap::new(),
                payload: Cursor::new(Vec::new()),
            }
        }

        pub fn incorrect_login() -> Self{
            let headers = maplit::hashmap!{
                "WWW-Authenticate".to_owned() => "Basic realm=\"simple\"".to_owned(),
                "Content-Length".to_owned() => "0".to_owned(),
            };

            Self{
                status: Status::Unauthorized,
                headers: headers,
                payload: Cursor::new(Vec::new()),
            }
        }
    }

    impl<S: AsyncRead + Unpin> Response<S> {
        pub fn status_and_headers(&self) -> String {
            let headers = self
                .headers
                .iter()
                .map(|(k, v)| format!("{k}: {v}"))
                .collect::<Vec<_>>()
                .join("\r\n");
            format!("HTTP/1.1 {}\r\n{headers}\r\n\r\n", self.status)
        }

        pub async fn write<O: AsyncWrite + Unpin>(
            mut self,
            stream: &mut O,
        ) -> Result<(), anyhow::Error> {
            stream
                .write_all(self.status_and_headers().as_bytes())
                .await?;

            tokio::io::copy(&mut self.payload, stream).await?;

            Ok(())
        }
    }

    #[derive(PartialEq, Debug, Clone, Copy)]
    pub enum Status {
        InternalError,
        NotFound,
        Ok,
        Unauthorized,
    }

    impl std::fmt::Display for Status {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::NotFound => write!(f, "404 Not Found"),
                Self::Ok => write!(f, "200 Ok"),
                Self::InternalError => write!(f, "505 Internal Error"),
                Self::Unauthorized => write!(f, "401 Unauthorized"),
            }
        }
    }

    #[derive(Debug)]
    pub enum ContentType {
        js,
        html,
        css,
        jpg,
        png,
        json,
        webp,
        ico,
    }

    impl ToString for ContentType {
        fn to_string(&self) -> String {
            match self {
                ContentType::js => "text/javascript".to_owned(),
                ContentType::html => "text/html".to_owned(),
                ContentType::css => "text/css".to_owned(),
                ContentType::jpg => "image/jpg".to_owned(),
                ContentType::png => "image/png".to_owned(),
                ContentType::json => "application/json".to_owned(),
                ContentType::webp => "image/webp".to_owned(),
                ContentType::ico => "image/x-icon".to_owned(),
            }
        }
    }

    impl FromStr for ContentType {
        type Err = anyhow::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let pos = s.chars().count() - s.chars().rev().position(|c| c == '.').unwrap() - 1;
            let file_type = &s[pos..];

            match file_type {
                ".html" => Ok(ContentType::html),
                ".css" => Ok(ContentType::css),
                ".js" => Ok(ContentType::js),
                ".png" => Ok(ContentType::png),
                ".jpg" => Ok(ContentType::jpg),
                ".json" => Ok(ContentType::json),
                ".webp" => Ok(ContentType::webp),
                ".ico" => Ok(ContentType::ico),
                _ => Err(anyhow::anyhow!("Invalid file type: {}", file_type)),
            }
        }
    }

    impl ContentType{

        pub fn from_path(s: &str) -> Result<Self, anyhow::Error> {
            let pos = s.chars().count() - s.chars().rev().position(|c| c == '.').unwrap() - 1;
            let file_type = &s[pos..];

            match file_type {
                ".html" => Ok(ContentType::html),
                ".css" => Ok(ContentType::css),
                ".js" => Ok(ContentType::js),
                ".png" => Ok(ContentType::png),
                ".jpg" => Ok(ContentType::jpg),
                ".json" => Ok(ContentType::json),
                ".webp" => Ok(ContentType::webp),
                ".ico" => Ok(ContentType::ico),
                _ => Err(anyhow::anyhow!("Invalid file type: {}", file_type)),
            }
        }
    }
}

//DATABASE URL
// const DB_URL: &str = env!("DATABASE_URL");

//constants
const OK_RESPONSE: &str =
    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, PUT, DELETE\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n";
const NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
const INTERNAL_ERROR: &str = "HTTP/1.1 500 INTERNAL ERROR\r\n\r\n";

//db setup
async fn set_database() -> Result<(), PostgresError> {
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres", NoTls).await?;
    client.batch_execute(
        "
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            display_name VARCHAR NOT NULL,
            email VARCHAR NOT NULL
        )
    "
    ).await?;
    Ok(())
}

//Get id from request URL
fn get_id(request: &str) -> &str {
    request.split("/").nth(4).unwrap_or_default().split_whitespace().next().unwrap_or_default()
}

//deserialize user from request body without id
fn get_user_request_body(request: &str) -> Result<User, serde_json::Error> {
    serde_json::from_str(request.split("\r\n\r\n").last().unwrap_or_default())
}

// Email address regex: "^[\w-\.]+@([\w-]+\.)+[\w-]{2,4}$"