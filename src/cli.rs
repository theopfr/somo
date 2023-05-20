use clap::Parser;


#[derive(Debug)]
pub struct FlagValues {
    pub check: bool,
    pub kill: u32,
    pub proto: Option<String>,
    pub ip: Option<String>,
    pub port: Option<String>,
    pub program: Option<String>,
    pub pid: Option<String>,
    pub open: bool,
    pub exclude_ipv6: bool
}


/*
pub by_proto: Option<String>,
pub by_program: Option<String>,
pub by_pid: Option<String>,
pub by_remote_address: Option<String>,
pub by_remote_port: Option<String>,
pub exclude_ipv6: bool
*/


/// get flags
#[derive(Parser, Debug)] 
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'c', long, default_value_t = false)]
    check: bool,

    #[arg(short = 'k', long, default_value_t = 0)]
    kill: u32,

    #[arg(long, default_value = None)]
    proto: Option<String>,

    #[arg(long, default_value = None)]
    ip: Option<String>,

    #[arg(long, default_value = None)]
    port: Option<String>,

    #[arg(long, default_value = None)]
    program: Option<String>,

    #[arg(long, default_value = None)]
    pid: Option<String>,

    #[arg(short = 'o', long, default_value_t = false)]
    open: bool,

    #[arg(short = 'e', long, default_value_t = false)]
    exclude_ipv6: bool,
}

pub fn cli() -> FlagValues {
    let args = Args::parse();

    let flag_values = FlagValues {
        check: args.check,
        kill: args.kill,
        proto: args.proto,
        ip: args.ip,
        program: args.port,
        port: args.program,
        pid: args.pid,
        open: args.open,
        exclude_ipv6: args.exclude_ipv6
    };

    return flag_values;
}