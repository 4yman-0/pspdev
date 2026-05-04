#BINARY_NAME=gl
BINARY_NAME=music
#BINARY_NAME=anim
#BINARY_NAME=render-to-tex
# #BINARY_NAME=io
# #BINARY_NAME=clock-speed
# #BINARY_NAME=time
# #BINARY_NAME=wlan
#BINARY_NAME=private

PSPEMU=PPSSPPSDL
PSPEMUFLAGS= --escape-exit 

CARGOFLAGS=--target=mipsel-sony-psp -Zunstable-options -Zbuild-std=core,alloc

SHELL=sh

TARGET=target/mipsel-sony-psp

EBOOT = $(TARGET)/debug/$(BINARY_NAME).EBOOT.PBP
EBOOT_RELEASE = $(TARGET)/release/$(BINARY_NAME).EBOOT.PBP

default: auto

run: $(EBOOT)
	$(PSPEMU) $(PSPEMUFLAGS) ./$<

run_release: $(EBOOT_RELEASE)
	$(PSPEMU) $(PSPEMUFLAGS) ./$<

compile:
	cargo psp -p $(BINARY_NAME) 

compile_release:
	cargo psp -p $(BINARY_NAME) --release

auto: compile run

pkg: $(EBOOT)
	@cp $^ build/EBOOT.PBP

pkg_release: $(EBOOT_RELEASE)
	-@mkdir build
	@cp $^ build/EBOOT.PBP

clippy:
	@cargo clippy $(CARGOFLAGS) --no-deps -- 

.PHONY: default run run_release auto pkg pkg_release clippy compile compile_release
