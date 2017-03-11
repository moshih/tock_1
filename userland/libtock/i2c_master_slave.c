#include "tock.h"
#include "i2c_master_slave.h"

struct i2c_master_slave_data {
  bool fired;
  int callback_type;
  int length;
};

static struct i2c_master_slave_data result = { .fired = false };

// Internal callback for faking synchronous reads
static void i2c_master_slave_cb(int callback_type,
                                int length,
                                __attribute__ ((unused)) int unused,
                                void* ud) {
  struct i2c_master_slave_data* data = (struct i2c_master_slave_data*) ud;
  data->callback_type = callback_type;
  data->length = length;
  data->fired = true;
}


int i2c_master_slave_set_callback(subscribe_cb callback, void* callback_args) {
  return subscribe(DRIVER_NUM_I2CMASTERSLAVE, 0, callback, callback_args);
}

int i2c_master_slave_set_master_write_buffer(uint8_t* buffer, uint32_t len) {
  return allow(DRIVER_NUM_I2CMASTERSLAVE, 0, (void*) buffer, len);
}

int i2c_master_slave_set_master_read_buffer(uint8_t* buffer, uint32_t len) {
  return allow(DRIVER_NUM_I2CMASTERSLAVE, 1, (void*) buffer, len);
}

int i2c_master_slave_set_slave_read_buffer(uint8_t* buffer, uint32_t len) {
  return allow(DRIVER_NUM_I2CMASTERSLAVE, 2, (void*) buffer, len);
}

int i2c_master_slave_set_slave_write_buffer(uint8_t* buffer, uint32_t len) {
  return allow(DRIVER_NUM_I2CMASTERSLAVE, 3, (void*) buffer, len);
}

int i2c_master_slave_write(uint8_t address, uint8_t length) {
  uint32_t a = (((uint32_t) length) << 16) | address;
  return command(DRIVER_NUM_I2CMASTERSLAVE, 1, a);
}

int i2c_master_slave_read(uint16_t address, uint16_t len) {
  uint32_t a = (((uint32_t) len) << 16) | address;
  return command(DRIVER_NUM_I2CMASTERSLAVE, 2, a);
}

int i2c_master_slave_listen(void) {
  return command(DRIVER_NUM_I2CMASTERSLAVE, 3, 0);
}

int i2c_master_slave_set_slave_address(uint8_t address) {
  return command(DRIVER_NUM_I2CMASTERSLAVE, 6, address);
}

int i2c_master_slave_enable_slave_read(uint32_t len) {
  return command(DRIVER_NUM_I2CMASTERSLAVE, 4, len);
}

int i2c_master_slave_write_sync(uint8_t address, uint8_t len) {
  int err;
  result.fired = false;

  err = i2c_master_slave_set_callback(i2c_master_slave_cb, (void*) &result);
  if (err < 0) return err;

  err = i2c_master_slave_write(address, len);
  if (err < 0) return err;

  // Wait for the callback.
  yield_for(&result.fired);

  return result.length;
}

int i2c_master_slave_read_sync(uint16_t address, uint16_t len) {
  int err;
  result.fired = false;

  err = i2c_master_slave_set_callback(i2c_master_slave_cb, (void*) &result);
  if (err < 0) return err;

  err = i2c_master_slave_read(address, len);
  if (err < 0) return err;

  // Wait for the callback.
  yield_for(&result.fired);

  return result.length;
}
