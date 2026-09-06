fn main() -> std::io::Result<()> {
    let mut command = nyintergroup_cli::command();

    if std::env::args_os().len() == 1 {
        command.print_help()?;
        println!();
        return Ok(());
    }

    let _matches = command.get_matches();
    Ok(())
}
