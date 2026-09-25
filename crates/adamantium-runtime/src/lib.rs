pub mod types;

use std::cell::{Cell, RefCell};
use std::ffi::{CStr, c_char};
use std::io::Write;
use types::{Type, Value};
use wasmi::{Config, Engine, Linker, Module, Store, StoreLimits, StoreLimitsBuilder};
use wasmi_wasi::{
    WasiCtx, WasiCtxBuilder, add_to_linker, ambient_authority, wasi_common::pipe::WritePipe,
};

thread_local! {
    static TRY_DEPTH: Cell<usize> = const { Cell::new(0) };
    static LAST_ERROR: RefCell<Option<RuntimeError>> = const { RefCell::new(None) };
}

const PACKAGE_FUEL: u64 = 10_000_000;
const MAX_PACKAGE_OUTPUT_BYTES: usize = 1024 * 1024;
const MAX_PACKAGE_MEMORY_BYTES: usize = 256 * 1024 * 1024;

struct PackageState {
    wasi: WasiCtx,
    limits: StoreLimits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeErrorKind {
    Recoverable,
    Panic,
    Package,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub code: &'static str,
    pub message: String,
    pub line: Option<usize>,
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.kind, self.line) {
            (RuntimeErrorKind::Recoverable, Some(line)) => {
                write!(
                    formatter,
                    "Adamantium runtime error at line {line}: {}",
                    self.message
                )
            }
            (RuntimeErrorKind::Recoverable, None) => {
                write!(formatter, "Adamantium runtime error: {}", self.message)
            }
            (RuntimeErrorKind::Panic, Some(line)) => {
                write!(
                    formatter,
                    "Adamantium program panicked at line {line}: {}",
                    self.message
                )
            }
            (RuntimeErrorKind::Panic, None) => {
                write!(formatter, "Adamantium program panicked: {}", self.message)
            }
            (RuntimeErrorKind::Package, _) => {
                write!(formatter, "Adamantium package error: {}", self.message)
            }
        }
    }
}

fn report_error(error: RuntimeError) {
    let handled = TRY_DEPTH.get() != 0;
    if handled {
        LAST_ERROR.with_borrow_mut(|last| *last = Some(error));
    } else {
        eprintln!("{error}");
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ad_try_begin() {
    if TRY_DEPTH.get() == 0 {
        LAST_ERROR.with_borrow_mut(|error| *error = None);
    }
    TRY_DEPTH.set(TRY_DEPTH.get() + 1);
}

/// # Safety
/// `output` must point to writable memory for one `Value`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ad_try_end(output: *mut Value) {
    TRY_DEPTH.set(TRY_DEPTH.get().saturating_sub(1));
    let Some(output) = (unsafe { output.as_mut() }) else {
        return;
    };
    let Some(error) = LAST_ERROR.with_borrow_mut(Option::take) else {
        *output = Value::default();
        return;
    };
    let bytes = error.to_string().into_bytes().into_boxed_slice();
    let value = Value {
        lo: bytes.as_ptr() as u64,
        hi: bytes.len() as u64,
    };
    std::mem::forget(bytes);
    *output = Value {
        lo: Box::into_raw(Box::new(value)) as u64,
        hi: 2,
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn ad_has_error() -> u32 {
    u32::from(LAST_ERROR.with_borrow(Option::is_some))
}

#[unsafe(no_mangle)]
pub extern "C" fn ad_is_trying() -> u32 {
    u32::from(TRY_DEPTH.get() != 0)
}

#[unsafe(no_mangle)]
pub extern "C" fn ad_object_new(field_count: usize) -> *mut Value {
    let fields = vec![Value::default(); field_count].into_boxed_slice();
    Box::into_raw(fields) as *mut Value
}

/// # Safety
/// `source` must point to at least `field_count` initialized values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ad_object_clone(source: *const Value, field_count: usize) -> *mut Value {
    if source.is_null() {
        return std::ptr::null_mut();
    }
    let fields = unsafe { std::slice::from_raw_parts(source, field_count) };
    Box::into_raw(fields.to_vec().into_boxed_slice()) as *mut Value
}

#[unsafe(no_mangle)]
pub extern "C" fn ad_list_error(index: usize, length: usize, line: usize) -> u32 {
    report_error(RuntimeError {
        kind: RuntimeErrorKind::Recoverable,
        code: "R001",
        message: format!("List index {index} is out of bounds for length {length}"),
        line: (line != 0).then_some(line),
    });
    2
}

#[cfg(test)]
mod list_tests {
    use super::*;

    #[test]
    fn allocates_zero_initialized_list_storage() {
        let list = ad_object_new(3);
        assert!(!list.is_null());
        let values = unsafe { std::slice::from_raw_parts(list, 3) };
        assert_eq!(values, &[Value::default(); 3]);
        unsafe { drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(list, 3))) };
    }

    #[test]
    fn clones_list_storage_independently() {
        let list = ad_object_new(2);
        unsafe {
            list.write(Value { lo: 10, hi: 0 });
            list.add(1).write(Value { lo: 20, hi: 0 });
        }
        let copy = unsafe { ad_object_clone(list, 2) };
        unsafe { copy.write(Value { lo: 99, hi: 0 }) };

        assert_eq!(unsafe { (*list).lo }, 10);
        assert_eq!(unsafe { (*copy).lo }, 99);
        assert_eq!(unsafe { (*copy.add(1)).lo }, 20);

        unsafe {
            drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(list, 2)));
            drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(copy, 2)));
        }
    }
}

#[cfg(test)]
mod runtime_error_tests {
    use super::*;

    #[test]
    fn recoverable_errors_keep_structured_context() {
        ad_try_begin();
        assert_eq!(ad_list_error(5, 2, 17), 2);
        let error = LAST_ERROR.with_borrow(|error| error.clone()).unwrap();
        assert_eq!(error.kind, RuntimeErrorKind::Recoverable);
        assert_eq!(error.code, "R001");
        assert_eq!(error.line, Some(17));
        assert_eq!(error.message, "List index 5 is out of bounds for length 2");
        assert_eq!(
            error.to_string(),
            "Adamantium runtime error at line 17: List index 5 is out of bounds for length 2"
        );
        TRY_DEPTH.set(0);
        LAST_ERROR.with_borrow_mut(Option::take);
    }

    #[test]
    fn panics_are_distinct_from_recoverable_errors() {
        let message = Value {
            lo: b"broken".as_ptr() as u64,
            hi: 6,
        };
        ad_try_begin();
        unsafe { ad_message(&message, 9, 1) };
        let error = LAST_ERROR.with_borrow(|error| error.clone()).unwrap();
        assert_eq!(error.kind, RuntimeErrorKind::Panic);
        assert_eq!(error.code, "P001");
        assert_eq!(error.line, Some(9));
        assert_eq!(
            error.to_string(),
            "Adamantium program panicked at line 9: broken"
        );
        TRY_DEPTH.set(0);
        LAST_ERROR.with_borrow_mut(Option::take);
    }
}

#[cfg(test)]
mod string_tests {
    use super::*;

    fn value(text: &str) -> Value {
        Value {
            lo: text.as_ptr() as u64,
            hi: text.len() as u64,
        }
    }

    fn request(operation: u32, a: Value, b: Value) -> Request {
        Request {
            a,
            b,
            c: Value::default(),
            output: Value::default(),
            operation,
            ty: Type::String.id(),
            from: Type::String.id(),
            reserved: 0,
        }
    }

    #[test]
    fn strings_use_unicode_scalar_lengths_and_indexes() {
        let text = "aŻ🙂";
        let length =
            unsafe { string_operation(&request(14, value(text), Value::default())) }.unwrap();
        assert_eq!(length.lo, 3);

        let indexed =
            unsafe { string_operation(&request(15, value(text), Value { lo: 2, hi: 0 })) }.unwrap();
        assert_eq!(unsafe { string_value(indexed) }.unwrap(), "🙂");
        assert!(
            unsafe { string_operation(&request(15, value(text), Value { lo: 3, hi: 0 })) }
                .unwrap_err()
                .contains("out of bounds")
        );
    }

    #[test]
    fn strings_compare_and_concatenate_by_value() {
        let left = "Ada";
        let right = "mantium";
        let joined = unsafe { string_operation(&request(0, value(left), value(right))) }.unwrap();
        assert_eq!(unsafe { string_value(joined) }.unwrap(), "Adamantium");
        unsafe {
            drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                joined.lo as *mut u8,
                joined.hi as usize,
            )));
        }

        let less = unsafe { string_operation(&request(9, value("abc"), value("bcd"))) }.unwrap();
        let equal = unsafe { string_operation(&request(7, value("żółw"), value("żółw"))) }.unwrap();
        assert_eq!(less.lo, 1);
        assert_eq!(equal.lo, 1);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ad_optional_error(line: usize) -> u32 {
    report_error(RuntimeError {
        kind: RuntimeErrorKind::Recoverable,
        code: "R002",
        message: "cannot access a field or method through None".into(),
        line: (line != 0).then_some(line),
    });
    2
}

#[repr(C)]
pub struct ArgumentSpec {
    pub name: *const u8,
    pub name_len: usize,
    pub ty: u32,
    pub optional: u32,
}

/// # Safety
/// `argv`, `specs`, and `output` must point to arrays described by their counts.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ad_parse_arguments(
    argc: usize,
    argv: *const *const c_char,
    specs: *const ArgumentSpec,
    count: usize,
    output: *mut Value,
) -> u32 {
    let argv = unsafe { std::slice::from_raw_parts(argv, argc) };
    let specs = unsafe { std::slice::from_raw_parts(specs, count) };
    let output = unsafe { std::slice::from_raw_parts_mut(output, count) };
    output.fill(Value::default());
    let mut seen = vec![false; count];
    let mut index = 1;
    while index < argv.len() {
        let argument = unsafe { CStr::from_ptr(argv[index]) }.to_string_lossy();
        let Some(name) = argument.strip_prefix("--") else {
            eprintln!("Adamantium argument warning: unknown argument '{argument}'");
            index += 1;
            continue;
        };
        let found = specs.iter().position(|spec| {
            let bytes = unsafe { std::slice::from_raw_parts(spec.name, spec.name_len) };
            bytes == name.as_bytes()
        });
        let Some(slot) = found else {
            eprintln!("Adamantium argument warning: unknown argument '--{name}'");
            index += 1;
            if index < argv.len() {
                let next = unsafe { CStr::from_ptr(argv[index]) }.to_bytes();
                if !next.starts_with(b"--") {
                    index += 1;
                }
            }
            continue;
        };
        if index + 1 >= argv.len() {
            eprintln!("Adamantium argument warning: missing value for '--{name}'");
            index += 1;
            continue;
        }
        let bytes = unsafe { CStr::from_ptr(argv[index + 1]) }.to_bytes();
        if bytes.starts_with(b"--") {
            eprintln!("Adamantium argument warning: missing value for '--{name}'");
            index += 1;
            continue;
        }
        let text = match std::str::from_utf8(bytes) {
            Ok(text) => text,
            Err(_) => return 2,
        };
        let declared = Type::from_id(specs[slot].ty).unwrap();
        let inner = if let Type::Optional(inner) = declared {
            Type::from_id(inner).unwrap()
        } else {
            declared
        };
        let value = if inner == Type::String {
            Value {
                lo: bytes.as_ptr() as u64,
                hi: bytes.len() as u64,
            }
        } else if inner == Type::Bool {
            match text {
                "true" => Value { lo: 1, hi: 0 },
                "false" => Value::default(),
                _ => {
                    eprintln!("Adamantium argument error: '--{name}' expects bool, found '{text}'");
                    return 2;
                }
            }
        } else {
            match types::literal(text, inner) {
                Ok(value) => value,
                Err(error) => {
                    eprintln!(
                        "Adamantium argument error: invalid value '{text}' for '--{name}' ({inner}): {error}"
                    );
                    return 2;
                }
            }
        };
        output[slot] = if declared == inner {
            value
        } else {
            types::convert(value, inner, declared).unwrap()
        };
        seen[slot] = true;
        index += 2;
    }
    for (slot, spec) in specs.iter().enumerate() {
        if !seen[slot] && spec.optional == 0 {
            let name = unsafe { std::slice::from_raw_parts(spec.name, spec.name_len) };
            eprintln!(
                "Adamantium argument warning: missing required argument '--{}'",
                String::from_utf8_lossy(name)
            );
        }
    }
    0
}

#[repr(C)]
pub struct Request {
    pub a: Value,
    pub b: Value,
    pub c: Value,
    pub output: Value,
    pub operation: u32,
    pub ty: u32,
    pub from: u32,
    pub reserved: u32,
}

unsafe fn string_value(value: Value) -> Result<&'static str, String> {
    if value.lo == 0 {
        return if value.hi == 0 {
            Ok("")
        } else {
            Err("invalid string pointer".into())
        };
    }
    let bytes = unsafe { std::slice::from_raw_parts(value.lo as *const u8, value.hi as usize) };
    std::str::from_utf8(bytes).map_err(|_| "string contains invalid UTF-8".into())
}

unsafe fn string_operation(request: &Request) -> Result<Value, String> {
    let left = unsafe { string_value(request.a)? };
    match request.operation {
        0 => {
            let right = unsafe { string_value(request.b)? };
            let bytes = [left.as_bytes(), right.as_bytes()]
                .concat()
                .into_boxed_slice();
            let value = Value {
                lo: bytes.as_ptr() as u64,
                hi: bytes.len() as u64,
            };
            std::mem::forget(bytes);
            Ok(value)
        }
        7..=12 => {
            let right = unsafe { string_value(request.b)? };
            let ordering = left.cmp(right);
            let result = match request.operation {
                7 => ordering.is_eq(),
                8 => !ordering.is_eq(),
                9 => ordering.is_lt(),
                10 => !ordering.is_gt(),
                11 => ordering.is_gt(),
                12 => !ordering.is_lt(),
                _ => unreachable!(),
            };
            Ok(Value {
                lo: result as u64,
                hi: 0,
            })
        }
        14 => Ok(Value {
            lo: left.chars().count() as u64,
            hi: 0,
        }),
        15 => {
            let index = usize::try_from(request.b.lo).map_err(|_| "string index is too large")?;
            let Some((start, character)) = left.char_indices().nth(index) else {
                return Err(format!(
                    "String index {index} is out of bounds for length {}",
                    left.chars().count()
                ));
            };
            Ok(Value {
                lo: unsafe { (request.a.lo as *const u8).add(start) } as u64,
                hi: character.len_utf8() as u64,
            })
        }
        _ => Err("invalid string operation".into()),
    }
}

/// # Safety
/// `request` must point to an initialized, writable Request owned by the caller.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ad_evaluate(request: *mut Request) -> u32 {
    let Some(request) = (unsafe { request.as_mut() }) else {
        return 2;
    };
    let Some(ty) = Type::from_id(request.ty) else {
        return 2;
    };
    let result = if request.operation == 5 {
        let Some(from) = Type::from_id(request.from) else {
            return 2;
        };
        types::convert(request.a, from, ty)
    } else if request.from == Type::String.id() || request.ty == Type::String.id() {
        unsafe { string_operation(request) }
    } else {
        types::operation(request.operation, ty, request.a, request.b, request.c)
    };
    match result {
        Ok(value) => {
            request.output = value;
            0
        }
        Err(error) => {
            report_error(RuntimeError {
                kind: RuntimeErrorKind::Recoverable,
                code: "R003",
                message: error,
                line: (request.reserved != 0).then_some(request.reserved as usize),
            });
            2
        }
    }
}

/// # Safety
/// `value` must point to an initialized Value. String pointers must reference
/// `hi` readable UTF-8 bytes for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ad_print(value: *const Value, ty: u32, newline: u32) -> u32 {
    let Some(value) = (unsafe { value.as_ref() }) else {
        return 1;
    };
    let Some(ty) = Type::from_id(ty) else {
        return 1;
    };
    let rendered;
    let bytes = if ty == Type::String {
        if value.lo == 0 && value.hi == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(value.lo as *const u8, value.hi as usize) }
        }
    } else {
        rendered = match types::display(*value, ty) {
            Ok(text) => text,
            Err(_) => return 1,
        };
        rendered.as_bytes()
    };
    let mut stdout = std::io::stdout().lock();
    if stdout.write_all(bytes).is_err()
        || (newline != 0 && stdout.write_all(b"\r\n").is_err())
        || stdout.flush().is_err()
    {
        1
    } else {
        0
    }
}

#[repr(C)]
pub struct PackageCall {
    wasm: Value,
    command: Value,
    arguments: [Value; 8],
    types: [u32; 8],
    count: u32,
    result_type: u32,
    filesystem: u32,
    reserved: u32,
    output: Value,
}

unsafe fn value_text(value: Value) -> Result<String, String> {
    if value.lo == 0 && value.hi == 0 {
        return Ok(String::new());
    }
    let bytes = unsafe { std::slice::from_raw_parts(value.lo as *const u8, value.hi as usize) };
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|error| format!("invalid UTF-8: {error}"))
}

fn package_result(bytes: &[u8], ty: Type) -> Result<Value, String> {
    if ty == Type::None {
        return Ok(Value::default());
    }
    if ty == Type::String {
        let bytes = bytes.to_vec().into_boxed_slice();
        let value = Value {
            lo: bytes.as_ptr() as u64,
            hi: bytes.len() as u64,
        };
        std::mem::forget(bytes);
        return Ok(value);
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|error| format!("package returned non-UTF-8 output: {error}"))?
        .trim();
    if ty == Type::Bool {
        return match text {
            "true" => Ok(Value { lo: 1, hi: 0 }),
            "false" => Ok(Value::default()),
            _ => Err(format!("package returned invalid bool '{text}'")),
        };
    }
    types::literal(text, ty)
}

/// # Safety
/// `request` must point to an initialized, writable `PackageCall` and all
/// contained string values must remain readable for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ad_package_call(request: *mut PackageCall) -> u32 {
    let Some(request) = (unsafe { request.as_mut() }) else {
        return 2;
    };
    let result = (|| -> Result<Value, String> {
        if request.count as usize > request.arguments.len() {
            return Err("package call has too many arguments".into());
        }
        if request.filesystem > 2 {
            return Err("package call has an invalid filesystem permission".into());
        }
        let wasm = unsafe { value_text(request.wasm)? };
        let command = unsafe { value_text(request.command)? };
        let mut arguments = vec!["adamantium-packet".to_owned(), command];
        for index in 0..request.count as usize {
            let ty = Type::from_id(request.types[index]).ok_or("invalid package argument type")?;
            let value = request.arguments[index];
            arguments.push(if ty == Type::String {
                unsafe { value_text(value)? }
            } else {
                types::display(value, ty)?
            });
        }

        let stdout = WritePipe::new_in_memory();
        let stderr = WritePipe::new_in_memory();
        let mut builder = WasiCtxBuilder::new();
        builder
            .args(&arguments)
            .map_err(|error| error.to_string())?
            .stdout(Box::new(stdout.clone()))
            .stderr(Box::new(stderr.clone()));
        if request.filesystem != 0 {
            let directory = wasmi_wasi::Dir::open_ambient_dir(
                std::env::current_dir().map_err(|error| error.to_string())?,
                ambient_authority(),
            )
            .map_err(|error| error.to_string())?;
            builder
                .preopened_dir(directory, ".")
                .map_err(|error| error.to_string())?;
        }
        let mut config = Config::default();
        config.consume_fuel(true);
        let engine = Engine::new(&config);
        let bytes = std::fs::read(&wasm).map_err(|error| error.to_string())?;
        adamantium_wasm::validate_package(&bytes)?;
        let module = Module::new(&engine, &bytes).map_err(|error| error.to_string())?;
        let mut linker: Linker<PackageState> = Linker::new(&engine);
        add_to_linker(&mut linker, |context| &mut context.wasi)
            .map_err(|error| error.to_string())?;
        let limits = StoreLimitsBuilder::new()
            .memory_size(MAX_PACKAGE_MEMORY_BYTES)
            .memories(1)
            .tables(8)
            .instances(8)
            .trap_on_grow_failure(true)
            .build();
        let mut store = Store::new(
            &engine,
            PackageState {
                wasi: builder.build(),
                limits,
            },
        );
        store.limiter(|state| &mut state.limits);
        store
            .set_fuel(PACKAGE_FUEL)
            .map_err(|error| format!("could not set package execution limit: {error}"))?;
        let execution = linker
            .instantiate_and_start(&mut store, &module)
            .and_then(|instance| {
                instance
                    .get_typed_func::<(), ()>(&store, "_start")?
                    .call(&mut store, ())
            });
        if let Err(error) = execution
            && error.i32_exit_status() != Some(0)
        {
            drop(store);
            let stderr = stderr
                .try_into_inner()
                .map_err(|_| "could not read package stderr".to_owned())?
                .into_inner();
            let stderr = String::from_utf8_lossy(&stderr).into_owned();
            return Err(if stderr.trim().is_empty() {
                error.to_string()
            } else {
                stderr
            });
        }
        drop(store);
        let stdout = stdout
            .try_into_inner()
            .map_err(|_| "could not read package stdout".to_owned())?
            .into_inner();
        if stdout.len() > MAX_PACKAGE_OUTPUT_BYTES {
            return Err(format!(
                "package output is {} bytes; the limit is {MAX_PACKAGE_OUTPUT_BYTES} bytes",
                stdout.len()
            ));
        }
        let result_type =
            Type::from_id(request.result_type).ok_or("invalid package result type")?;
        package_result(&stdout, result_type)
    })();
    match result {
        Ok(value) => {
            request.output = value;
            0
        }
        Err(error) => {
            report_error(RuntimeError {
                kind: RuntimeErrorKind::Package,
                code: "R004",
                message: error.trim().into(),
                line: None,
            });
            2
        }
    }
}

/// # Safety
/// `message` must point to a string `Value` whose pointer and length are valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ad_message(message: *const Value, line: usize, panic: u32) {
    let Some(message) = (unsafe { message.as_ref() }) else {
        return;
    };
    let bytes = unsafe { std::slice::from_raw_parts(message.lo as *const u8, message.hi as usize) };
    let text = String::from_utf8_lossy(bytes);
    if panic != 0 {
        report_error(RuntimeError {
            kind: RuntimeErrorKind::Panic,
            code: "P001",
            message: text.into_owned(),
            line: Some(line),
        });
    } else {
        eprintln!("Adamantium program warned at line {line}: {text}");
    }
}

#[cfg(test)]
mod package_tests {
    use super::*;

    fn text_value(value: &str) -> Value {
        Value {
            lo: value.as_ptr() as u64,
            hi: value.len() as u64,
        }
    }

    #[test]
    fn invokes_a_wasi_package_and_converts_its_result() {
        let directory =
            std::env::temp_dir().join(format!("adamantium-wasm-abi-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("answer.wasm");
        std::fs::write(
            &path,
            wat::parse_str(
                r#"(module
                    (import "wasi_snapshot_preview1" "fd_write"
                        (func $fd_write (param i32 i32 i32 i32) (result i32)))
                    (memory (export "memory") 1)
                    (data (i32.const 16) "42")
                    (func (export "_start")
                        (i32.store (i32.const 0) (i32.const 16))
                        (i32.store (i32.const 4) (i32.const 2))
                        (drop (call $fd_write
                            (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 8)))))"#,
            )
            .unwrap(),
        )
        .unwrap();
        let wasm = path.to_string_lossy();
        let command = "answer";
        let mut request = PackageCall {
            wasm: text_value(&wasm),
            command: text_value(command),
            arguments: [Value::default(); 8],
            types: [0; 8],
            count: 0,
            result_type: Type::I32.id(),
            filesystem: 0,
            reserved: 0,
            output: Value::default(),
        };

        assert_eq!(unsafe { ad_package_call(&mut request) }, 0);
        assert_eq!(request.output.lo as i64, 42);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn stops_packages_that_exhaust_the_execution_budget() {
        let directory =
            std::env::temp_dir().join(format!("adamantium-wasm-fuel-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("loop.wasm");
        std::fs::write(
            &path,
            wat::parse_str(
                r#"(module
                    (func (export "_start")
                        (loop $forever (br $forever))))"#,
            )
            .unwrap(),
        )
        .unwrap();
        let wasm = path.to_string_lossy();
        let mut request = PackageCall {
            wasm: text_value(&wasm),
            command: text_value("loop"),
            arguments: [Value::default(); 8],
            types: [0; 8],
            count: 0,
            result_type: Type::None.id(),
            filesystem: 0,
            reserved: 0,
            output: Value::default(),
        };

        assert_eq!(unsafe { ad_package_call(&mut request) }, 2);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn converts_all_supported_result_shapes_and_rejects_invalid_values() {
        assert_eq!(package_result(b"true\n", Type::Bool).unwrap().lo, 1);
        assert_eq!(package_result(b"false", Type::Bool).unwrap().lo, 0);
        assert!(package_result(b"yes", Type::Bool).is_err());
        assert_eq!(package_result(b"", Type::None).unwrap(), Value::default());
        let string = package_result(b"text\n", Type::String).unwrap();
        assert_eq!(unsafe { value_text(string) }.unwrap(), "text\n");
    }
}
