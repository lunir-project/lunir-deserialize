use std::rc::Rc;
use std::{collections::HashMap, ops::Index};

use lunir::prelude::{Constant, Value, Vararg};

use crate::bytecode::{Instruction, LuauChunk, LuauChunkBuilder};

struct Deserializer<T: AsRef<[u8]>> {
    buffer: T,
    read_position: usize,

    nested_strings: Vec<String>,
    nested_protos: HashMap<usize, Rc<LuauChunk>>,
}

trait FromSlice: Sized {
    fn from_sliced(slice: &[u8]) -> Self;
}

impl FromSlice for i32 {
    fn from_sliced(slice: &[u8]) -> Self {
        let (int_bytes, _) = slice.split_at(std::mem::size_of::<i32>());
        let int = i32::from_le_bytes(int_bytes.try_into().unwrap());

        int
    }
}

impl FromSlice for u32 {
    fn from_sliced(slice: &[u8]) -> Self {
        let (int_bytes, _) = slice.split_at(std::mem::size_of::<u32>());
        let int = u32::from_le_bytes(int_bytes.try_into().unwrap());

        int
    }
}

impl FromSlice for bool {
    fn from_sliced(slice: &[u8]) -> Self {
        slice[0] != 0
    }
}

impl FromSlice for f64 {
    fn from_sliced(slice: &[u8]) -> Self {
        let (float_bytes, _) = slice.split_at(std::mem::size_of::<f64>());
        let float = f64::from_le_bytes(float_bytes.try_into().unwrap());

        float
    }
}

impl FromSlice for u8 {
    fn from_sliced(slice: &[u8]) -> Self {
        slice[0]
    }
}

impl FromSlice for i8 {
    fn from_sliced(slice: &[u8]) -> Self {
        slice[0] as i8
    }
}

#[derive(Debug)]
enum DeserializerError {
    InvalidVersion,
    CompilerError(String),
}

impl<T: AsRef<[u8]>> Deserializer<T> {
    fn next<R: FromSlice>(&mut self) -> R {
        let result = R::from_sliced(&self.buffer.as_ref()[self.read_position..]);

        self.read_position += std::mem::size_of::<R>();

        result
    }

    fn read_compressed_int(&mut self) -> usize {
        let mut result = 0_usize;
        let mut size = 0;

        let mut current = 0_i8;

        loop {
            current = self.next::<i8>();
            result |= ((current & 0x7F) << size) as usize;
            size += 7;

            if current < 0 {
                break;
            }
        }

        result
    }

    fn read_string(&mut self) -> String {
        let size = self.read_compressed_int();
        let mut result = String::with_capacity(size);

        for _ in 0..size {
            result.push(self.next::<u8>() as char);
        }

        result
    }

    fn read_proto(&mut self) -> Rc<LuauChunk> {
        let mut result = LuauChunkBuilder::default();

        result
            .max_stack_size(self.next())
            .num_params(self.next())
            .nups(self.next())
            .is_vararg(match self.next::<u8>() {
                1 => Vararg::HasArg,
                2 => Vararg::IsVararg,
                4 => Vararg::NeedsArg,
                _ => panic!("invalid vararg flag"),
            });

        let size_code = self.read_compressed_int();
        let mut instructions = Vec::with_capacity(size_code);

        for _ in 0..size_code {
            instructions.push(Instruction(self.next::<u32>()));
        }

        result.instructions(instructions);

        let size_constants = self.read_compressed_int();
        let mut constants = Vec::with_capacity(size_constants);

        for _ in 0..size_constants {
            let kind = self.next::<u8>();

            match kind {
                0 => constants.push(Constant::Nil),
                1 => constants.push(Constant::Boolean(self.next())),
                2 => constants.push(Constant::Number(self.next())),
                3 => {
                    let index = self.read_compressed_int() - 1;

                    constants.push(Constant::String(self.nested_strings[index].clone()));
                }

                // import type constant isn't needed to lift, lets skip it.
                4 => {
                    self.read_position += 4;
                }

                5 => {
                    let keys = self.read_compressed_int();
                    let mut table = Vec::<Value>::with_capacity(keys);

                    for _ in 0..keys {
                        let constant_index = self.read_compressed_int();
                        table.push(Value::ConstantIndex(constant_index));
                    }

                    constants.push(Constant::Table(lunir::il::Table::Array(table)));
                }

                // We must have an integer constant type in here so I can index our proto list.
                6 => {
                    let index = self.read_compressed_int();

                    todo!("closure constant, proto index: {}", index)
                }

                _ => panic!("unknown constant type: {}", kind),
            }
        }

        result.constants(constants);

        let sizep = self.read_compressed_int();
        let mut functions = Vec::with_capacity(sizep);

        for _ in 0..sizep {
            let index = self.read_compressed_int();

            functions.push(self.nested_protos.index(&index).clone());
        }

        result.functions(functions);

        let name_index = self.read_compressed_int();
        result.name(if name_index > 0 {
            Some(self.nested_strings[name_index - 1].clone())
        } else {
            None
        });

        let lineinfo_enabled = self.next::<bool>();
        if lineinfo_enabled {
            let compression_shift = self.next::<u8>();
            let size = (((size_code - 1) >> compression_shift) as usize) + 1;
            let mut lineinfo = Vec::with_capacity(size);

            self.read_position += size_code;

            for _ in 0..size {
                lineinfo.push(self.next::<u32>() as usize);
            }
        }

        // skip debug info for now.
        let debug_enabled = self.next::<bool>();
        if debug_enabled {
            todo!("deserialize debugging information");
        }

        Rc::new(result.build().unwrap())
    }

    pub fn with_buffer(buffer: T) -> Self {
        Self {
            buffer,
            read_position: 0,
            nested_protos: HashMap::new(),
            nested_strings: Vec::new(),
        }
    }

    pub fn deserialize(&mut self) -> Result<Rc<LuauChunk>, DeserializerError> {
        let version = self.next::<u8>();

        if version == 0 {
            let size = self.buffer.as_ref().len() - 1;
            let mut error = String::with_capacity(size);

            for _ in 0..size {
                error.push(self.next::<u8>() as char);
            }

            return Err(DeserializerError::CompilerError(error));
        }

        if version < 3 {
            return Err(DeserializerError::InvalidVersion);
        }

        let string_table_size = self.read_compressed_int();
        self.nested_strings.reserve(string_table_size);

        for _ in 0..string_table_size {
            let string = self.read_string();
            self.nested_strings.push(string);
        }

        let proto_size = self.read_compressed_int();
        self.nested_protos.reserve(proto_size);

        for _ in 0..proto_size {
            let chunk = self.read_proto();
            self.nested_protos
                .insert(self.read_position, Rc::clone(&chunk));
        }

        let main_proto = self.read_compressed_int();
        Ok(self.nested_protos.index(&main_proto).clone())
    }
}