CC = gcc
CFLAGS = -Iinclude

all: libfile_manager.a

libfile_manager.a: c_src/file_manager.o
	ar rcs libfile_manager.a c_src/file_manager.o

c_src/file_manager.o: c_src/file_manager.c
	$(CC) $(CFLAGS) -c c_src/file_manager.c -o c_src/file_manager.o

clean:
	rm -f c_src/*.o libfile_manager.a
