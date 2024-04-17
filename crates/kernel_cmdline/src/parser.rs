use core::str::FromStr;

use crate::KernelCommandLine;

struct OptionalKeyValuePair<'a> {
    pub key: &'a str,
    pub value: Option<&'a str>,
}

impl<'a> OptionalKeyValuePair<'a> {
    pub fn new(key: &'a str, value: Option<&'a str>) -> Self {
        Self { key, value }
    }

    pub fn from_line(line: &'a str) -> Self {
        let idx = line.find('=');

        match idx {
            Some(idx) => {
                let (key, value) = line.split_at(idx);

                assert!(value.chars().nth(0) == Some('='));

                let substr = &value[1..value.len()];
                Self::new(key, Some(substr))
            }
            None => Self::new(line, None),
        }
    }

    #[allow(dead_code)]
    pub fn get_str(&self) -> Option<&'a str> {
        self.value
    }

    pub fn get<T: FromStr>(&self) -> Option<T> {
        self.value.and_then(|str| str.parse().ok())
    }
}

pub struct KernelCommandLineParser<'a> {
    data: &'a str,
}

impl<'a> KernelCommandLineParser<'a> {
    pub fn new(data: &'a str) -> Self {
        Self { data }
    }

    fn keyvalue_pairs(&self) -> impl Iterator<Item = OptionalKeyValuePair<'a>> {
        self.data
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| OptionalKeyValuePair::from_line(line))
    }

    pub fn parse(&self) -> KernelCommandLine {
        let mut welcome = None;
        let mut use_reloc = None;
        let mut stack_size = None;
        let mut initial_heap_size = None;

        for keyvalue in self.keyvalue_pairs() {
            if keyvalue.key == "welcome" {
                welcome = Some(());
            }

            if keyvalue.key == "use_reloc" {
                use_reloc = keyvalue.get();
            }

            if keyvalue.key == "stack_size" {
                stack_size = keyvalue.get();
            }

            if keyvalue.key == "initial_heap_size" {
                initial_heap_size = keyvalue.get();
            }
        }

        let cmd = KernelCommandLine {
            welcome,
            use_reloc,
            stack_size,
            initial_heap_size,
        };

        cmd.verfy();

        cmd
    }
}
