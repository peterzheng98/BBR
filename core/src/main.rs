use std::time::SystemTime;

fn main(){
    let at = SystemTime::now();
    println!("hello world");
    let mut a : i32 = 1;
    let mut b : i32 = a;
    let mut c : i32 = a;
    b = b + c;
    a = a + c;
    c = 2;
    println!("{}", a);
    println!("{}", b);
    println!("{}", c);
    let b2 = at.duration_since(at).expect("111");
    println!("{:?}", b2);

}