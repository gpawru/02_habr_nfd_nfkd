#[macro_export]
/// записать кодпоинт
macro_rules! write {
    ($result: expr, $($code: expr),+) => {{
        $(
            $result.push(unsafe { char::from_u32_unchecked($code) });
        )+
    }};
}

// в expansions кодпоинт и его CCC записаны в 32-битном формате - старший байт - CCC, остальные - код

#[macro_export]
/// прочитать CCC из старшего байта u32
macro_rules! c32_ccc {
    ($entry: expr) => {
        unsafe { *(($entry as *const u32 as *const u8).add(3)) }
    };
}

#[macro_export]
/// прочитать кодпоинт из u32, где старший байт - CCC
macro_rules! c32_code {
    ($entry: expr) => {
        $entry & 0x3FFFF
    };
}

#[macro_export]
/// разбираем u64 на составляющие: o!(исходный u64, тип результата <T>, (опционально: смещение в <T>))
macro_rules! o {
    ($value: expr, $t: ty) => {
        unsafe { *(&$value as *const u64 as *const $t) }
    };
    ($value: expr, $t: ty, $offset: expr) => {
        unsafe { *((&$value as *const u64 as *const $t).add($offset)) }
    };
}
