#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn() -> Result<&'static str, &'static str>]) {
    println!("Running {} tests...", tests.len());
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
        "{} tests in total, {} succeed, {} failed",
        tests.len(),
        succeed,
        tests.len() - succeed
    );
    crate::sbi::shutdown();
}

#[macro_export]
macro_rules! test {
    ($func_name: ident, $func: block) => {
        #[test_case]
        fn $func_name() -> Result<&'static str, &'static str> {
            print!(
                "\x1b[4;37m{}\x1b[0m:\t{}...",
                file!(),
                stringify!($func_name)
            );
            $func
        }
    };
}

#[macro_export]
macro_rules! test_assert {
    ($assert_expr: expr, $info: literal) => {
        if !$assert_expr {
            return Err($info);
        }
    };
    ($assert_expr: expr) => {
        if !$assert_expr {
            return Err(stringify!($assert_expr));
        }
    };
}
