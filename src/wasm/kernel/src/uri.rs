use regex::Regex;

#[derive(Debug)]
pub struct Uri {
    scheme: String,
    connection_argument: Option<String>,
    host: String,
    port: Option<i32>,
    path: String,
    request_argument: Option<String>,
}

impl std::fmt::Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}://", self.scheme)?;
        if let Some(ref r) = self.connection_argument {
            write!(f, "{}@", r)?;
        }
        write!(f, "{}", self.host)?;
        if let Some(p) = self.port {
            write!(f, ":{}", p)?;
        }

        write!(f, "{}", self.path)?;
        if let Some(ref r) = self.request_argument {
            write!(f, "?{}", r)?;
        }
        Ok(())
    }
}

impl TryFrom<String> for Uri {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let r = Regex::new(
            r"(?x)
            ^
            (?P<scheme>([^:])+)://
            ((?P<connection_argument>[^@]+)@)?
            (?P<host>[^:/]+)
            (:(?P<port>\d+))?
            (?P<path>/[^?]+)
            (\?(?P<request_argument>.*))
            $
        ",
        );

        if let Err(e) = r {
            return Err(e.to_string());
        }
        let r = r.unwrap();

        let m = if let Some(m) = r.captures(&s) {
            m
        } else {
            return Err("Invalid URI".to_string());
        };

        let uri = Uri {
            scheme: m.name("scheme").unwrap().as_str().to_string(),
            connection_argument: m
                .name("connection_argument")
                .map(|x| x.as_str().to_string()),
            host: m.name("host").unwrap().as_str().to_string(),
            port: m.name("port").map(|x| x.as_str().parse().unwrap()),
            path: m.name("path").unwrap().as_str().to_string(),
            request_argument: m.name("request_argument").map(|x| x.as_str().to_string()),
        };

        Ok(uri)
    }
}
