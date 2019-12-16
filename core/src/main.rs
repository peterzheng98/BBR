fn main(){
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

}