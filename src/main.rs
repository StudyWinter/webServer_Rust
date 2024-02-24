use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use web_server::ThreadPool;



fn main() {
    // 监听本地端口为7878的进程
    // 使用unwrap()处理bind失败的结果
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    // 创建多线程
    let pool = ThreadPool::new(4);

    // incoming方法返回一个迭代器，提供一系列的流（代表客户端和服务端之间的打开的连接）
    // 即客户端连接服务端、服务端生成响应以及服务端关闭连接的全部请求 / 响应过程
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        // 执行
        pool.execute(|| {
            handle_connection(stream);
        });
    }

}


// 处理连接
// 参数是可变的
fn handle_connection(mut stream : TcpStream ) {
    // 从stream中读取数据并且存到buf_reader，注意这里传参数
    let buf_reader = BufReader::new(&mut stream);
    // 读取请求行
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    // 对request_line进行匹配，如果是GET / HTTP/1.1，则status_line = HTTP/1.1 200 OK， filename = hello.html
    // 否则status_line = HTTP/1.1 404 NOT FOUND， filename = 404.html
    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    // 读文件
    let contents = fs::read_to_string(filename).unwrap();
    let len = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {len}\r\n\r\n{contents}");

    // 把响应写回到客户端
    stream.write_all(response.as_bytes()).unwrap();
}

