#[allow(unused_imports)]
use crate::sbi::shutdown;

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn() -> Result<&'static str, &'static str>]) {
    println!("Running {} unit tests...", tests.len());
    let mut succeed = 0;
    for test in tests {
        let res = test();
        if res.is_ok() {
            println!(" \x1b[1;32m[ok]: \"{}\"\x1b[0m", res.unwrap());
            succeed += 1;
        } else {
            println!(" \x1b[1;31m[err]: \"{}\"\x1b[0m", res.unwrap_err());
        }
    }
    println!(
        "{} unit test in total, {} succeed, {} failed",
        tests.len(),
        succeed,
        tests.len() - succeed
    );
    shutdown();
}

#[macro_export]
macro_rules! unit_test {
    ($func_name: ident, $func: block) => {
        #[test_case]
        fn $func_name() -> Result<&'static str, &'static str> {
            print!("{}...", stringify!($func_name));
            $func
        }
    };
}

#[macro_export]
macro_rules! system_test {
    ($func_name: ident) => {
        #[cfg(feature = "system_test")]
        {
            print!("\x1b[1;33m");
            $func_name();
            print!("\x1b[0m");
        }
    };
}

unit_test!(test_do_math, {
    let mut a = 0;
    for i in 1..=10 {
        a += i;
    }
    if a == 55 {
        Ok("Genius")
    } else {
        Err("What's wrong with you")
    }
});

unit_test!(test_fucked_up, { Err("Sorry that I fucked it up") });
