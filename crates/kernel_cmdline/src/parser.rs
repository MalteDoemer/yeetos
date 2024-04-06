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
        let mut kernel_use_reloc = None;
        let mut kernel_stack_size = None;

        for keyvalue in self.keyvalue_pairs() {
            if keyvalue.key == "welcome" {
                welcome = Some(());
            }

            if keyvalue.key == "kernel_use_reloc" {
                kernel_use_reloc = keyvalue.get();
            }

            if keyvalue.key == "kernel_stack_size" {
                kernel_stack_size = keyvalue.get();
            }
        }

        KernelCommandLine {
            welcome,
            kernel_use_reloc,
            kernel_stack_size,
        }
    }
}
