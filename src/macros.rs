#[macro_export]
macro_rules! prometheus_help {
    ($str:literal) => {
        concat!(
            "#HELP ",
            $str,
            " ",
            env!("CARGO_PKG_NAME"),
            " v",
            env!("CARGO_PKG_VERSION")
        )
    };
}

#[macro_export]
macro_rules! prometheus_type {
    ($str:literal, $type: literal) => {
        concat!("#TYPE ", $str, " ", $type)
    };
}

#[macro_export]
macro_rules! write_prometheus_blurb {
    ($target:expr, $metric:literal, $type:literal) => {
        writeln!($target, crate::prometheus_help!($metric))?;
        writeln!($target, crate::prometheus_type!($metric, $type))?;
    };
}
