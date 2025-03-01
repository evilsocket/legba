use std::io::{Write, stdout};

use crate::session::Error;

pub(crate) fn read_arg_from_user(var_name: &str, default: Option<&str>) -> Result<String, Error> {
    let prompt = if let Some(def) = default {
        format!("{} ({}): ", var_name, def)
    } else {
        format!("{}: ", var_name)
    };

    loop {
        print!("recipe.{}", &prompt);
        let _ = stdout().flush();

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| e.to_string())?;

        let input = input.trim();
        if !input.is_empty() {
            return Ok(input.to_owned());
        } else if let Some(def) = default {
            return Ok(def.to_owned());
        }
        // keep going
    }
}
