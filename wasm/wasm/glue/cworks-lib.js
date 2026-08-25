mergeInto(LibraryManager.library, {
  cworks_js_callback__sig: 'iiii',
  cworks_js_callback: function(id, dataPtr, dataLen) {
    var data = UTF8ToString(dataPtr, dataLen);
    var result = Module._invokeJsCallback(id, data);
    if (result === null || result === undefined) {
      return -1;
    }
    var resultStr = String(result);
    var lengthBytes = lengthBytesUTF8(resultStr);
    var ptr = _malloc(lengthBytes + 1);
    stringToUTF8(resultStr, ptr, lengthBytes + 1);
    return ptr;
  },
  cworks_console_debug__sig: 'vi',
  cworks_console_debug: function(strPtr) {
    console.debug(UTF8ToString(strPtr));
  }
});
