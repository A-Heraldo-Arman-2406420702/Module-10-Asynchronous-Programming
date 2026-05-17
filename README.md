# Module 10 - Asynchronous Programming

## Experiment 1.2: Understanding how it works

![Screenshot Eksperimen 1.2](docs/images/first-screenshot.png)

**Penjelasan:**
Pada eksperimen ini, pesan `Heraldo's Komputer: hey hey` tercetak lebih dulu dibandingkan `howdy!` dan `done!`. 

Hal ini terjadi karena pemanggilan fungsi `spawner.spawn(...)` tidak langsung mengeksekusi *future* yang diberikan pada saat itu juga, melainkan hanya memasukkan *task* tersebut ke dalam antrean (task queue) dari *executor*. Setelah *task* dimasukkan ke antrean, alur program pada fungsi `main` (yang bersifat *synchronous*) terus berjalan ke baris selanjutnya, sehingga ia langsung mengeksekusi perintah `println!("Heraldo's Komputer: hey hey");`.

Barulah ketika baris terakhir `executor.run()` dieksekusi, *executor* akan memproses antrean *task* yang ada, menjalankan *future* yang tadi sudah di-*spawn* (mencetak `howdy!`), menunggu *timer* selama 2 detik, lalu mencetak `done!`.