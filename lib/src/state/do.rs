
    // pub fn append(&mut self, d: Dispatch) -> Result<()> {
    //     let (key, mut args) = d.pair();

    //     match self.get_key(key, Object::list) {
    //         Value::Bytes(ref mut bytes) => Self::_append_bytes(bytes, args),
    //         Value::List(ref mut list) => Self::_append_list(list, args),
    //         Value::String(ref mut string) => {
    //             Self::_append_str(string, args.remove(0))
    //         },
    //         _ => Err(DispatchError::KeyWrongType),
    //     }
    // }

//     #[inline]
//     pub fn append_bytes(&mut self, key: Vec<u8>, args: Vec<Vec<u8>>) -> Result<()> {
//         Self::_append_bytes(self.get_bytes(key)?, args)
//     }

//     #[inline]
//     pub fn _append_bytes(bytes: &mut Vec<u8>, args: Vec<Vec<u8>>) -> Result<()> {
//         args.iter().for_each(|arg| {
//             bytes.extend(arg);
//         });

//         Ok(())
//     }

//     #[inline]
//     pub fn append_list(&mut self, key: Vec<u8>, args: Vec<Vec<u8>>) -> Result<()> {
//         Self::_append_list(self.get_list(key)?, args)
//     }

//     #[inline]
//     fn _append_list(list: &mut Vec<Vec<u8>>, args: Vec<Vec<u8>>) -> Result<()> {
//         list.extend(args);

//         Ok(())
//     }

//     #[inline]
//     pub fn append_str(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
//         Self::_append_str(self.get_str(key)?, value)
//     }

//     fn _append_str(string: &mut String, value: Vec<u8>) -> Result<()> {
//         let lossy = String::from_utf8_lossy(&value);

//         match lossy {
//             Cow::Borrowed(v) => string.push_str(v),
//             Cow::Owned(v) => string.push_str(&v),
//         }

//         Ok(())
//     }

// //     pub fn decrement_by(&mut self, key: Vec<u8>, mut value: Vec<Vec<u8) -> Result<()> {
// //         match self.get_key(key, Object::float) {
// //             Value::Float(ref mut float) => Self::decrement_float_by(float, value[0]),
// //             Value::Integer(ref mut int) => Self::_decrement_int_by(int, value),
// //             _ => return Err(DispatchError::KeyWrongType),
// //         }
// //     }

//     pub fn decrement_float(&mut self, key: Vec<u8>) -> Result<()> {
//         Self::_decrement_float_by(self.get_float(key)?, 1.)
//     }

//     pub fn decrement_float_by(&mut self, key: Vec<u8>, arg: Vec<u8>) -> Result<()> {
//         let amount = utils::into_float(&arg)?;

//         Self::_decrement_float_by(self.get_float(key)?, amount)
//     }

//     fn _decrement_float_by(float: &mut f64, value: f64) -> Result<()> {
//         *float -= value;

//         Ok(())
//     }

//     pub fn decrement_int(&mut self, d: Dispatch) -> Result<i64> {
//         log::debug!("{:?}", d);
//         let key = d.key();

//         Self::_decrement_int_by(self.get_int(key)?, 1)
//     }

//     pub fn decrement_int_by(&mut self, key: Vec<u8>, bytes: Vec<u8>) -> Result<i64> {
//         let amount = utils::into_integer(&bytes)?;

//         Self::_decrement_int_by(self.get_int(key)?, amount)
//     }

//     #[inline]
//     fn _decrement_int_by(int: &mut i64, amount: i64) -> Result<i64> {
//         *int = int.saturating_sub(amount);

//         Ok(*int)
//     }

//      pub fn increment_int(&mut self, d: Dispatch) -> Result<i64> {
//          let key = d.key();

//          Self::_increment_int_by(self.get_int(key)?, 1)
//      }

//      pub fn increment_int_by(&mut self, key: Vec<u8>, amount: i64) -> Result<i64> {
//          Self::_increment_int_by(self.get_int(key)?, amount)
//      }

//      #[inline]
//      fn _increment_int_by(int: &mut i64, amount: i64) -> Result<i64> {
//          *int = int.saturating_add(amount);

//          Ok(*int)
//      }

//     #[inline]
//     pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
//         Self::_set(self.get_bytes(key)?, value)
//     }

//     #[inline]
//     fn _set(bytes: &mut Vec<u8>, value: Vec<u8>) -> Result<()> {
//         *bytes = value;

//         Ok(())
//     }

//     /// Sets a key's value to the provided boolean, regardless of whether the key
//     /// already exists.
//     ///
//     /// Casts the character '0' (`\x30`) to `false` and the character '1'
//     /// (`\x31`) to `true`.
//     ///
//     /// This is an O(1) operation.
//     ///
//     /// # Errors
//     ///
//     /// Returns [DispatchError::BooleanValueInvalid] if the argument given can
//     /// not be cast to a boolean.
//     #[inline]
//     pub fn set_boolean(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
//         Self::_set_boolean(self.get_bool(key)?, value)
//     }

//     fn _set_boolean(boolean: &mut bool, value: Vec<u8>) -> Result<()> {
//         *boolean = utils::into_boolean(value[0])?;

//         Ok(())
//     }

//     /// Returns the number of bytes contained in a string.
//     ///
//     /// This is an O(1) operation.
//     ///
//     /// *Note**: This returns the number of _bytes_, not the number of
//     /// _characters_.
//     ///
//     /// # Errors
//     ///
//     /// Returns [DispatchError::KeyNonexistant] if the key does not exist.
//     ///
//     /// Returns [DispatchError::KeyWrongType] if the key's value is not a
//     /// [Value::String].
//     #[inline]
//     pub fn strlen(&mut self, key: Vec<u8>) -> Result<usize> {
//         self.get_string(key).map(Self::_strlen)
//     }

//     #[inline]
//     fn _strlen(string: &mut String) -> usize {
//         string.len()
//     }

//     /// Returns the number of keys in the state.
//     ///
//     /// This is an O(1) operation.
//     ///
//     /// # Errors
//     ///
//     /// This always produces an OK value.
//     ///
//     #[inline]
//     pub fn system_keys(&mut self) -> usize {
//         self.0.len()
//     }
