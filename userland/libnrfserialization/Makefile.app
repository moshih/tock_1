LIBNRFSER_DIR := $(TOCK_USERLAND_BASE_DIR)/libnrfserialization

# So it doesn't think it's on the nRF and try to include nRF code.
CFLAGS += -D__TOCK__
CFLAGS += -DSVCALL_AS_NORMAL_FUNCTION
CFLAGS += -DSOFTDEVICE_s130

CFLAGS += -I$(LIBNRFSER_DIR)/headers

LIBS += $(LIBNRFSER_DIR)/libnrfserialization.a

# If environment variable V is non-empty, be verbose
ifneq ($(V),)
	Q=
	TRACE_TAR =
else
	Q=@
	TRACE_TAR = @echo " TAR       " $<
endif

.PHONY:	all
all: $(LIBNRFSER_DIR)/headers

$(LIBNRFSER_DIR)/headers: $(LIBNRFSER_DIR)/headers.tar.gz
	$(TRACE_TAR)
	$(Q)tar xf $< --directory $(LIBNRFSER_DIR)
	@touch $@
