# Module 10 - Asynchronous Programming

## Experiment 1.2: Understanding how it works

![Screenshot Eksperimen 1.2](docs/images/first-screenshot.png)

**Penjelasan:**
Pada eksperimen ini, pesan `Heraldo's Komputer: hey hey` tercetak lebih dulu dibandingkan `howdy!` dan `done!`. 

Hal ini terjadi karena pemanggilan fungsi `spawner.spawn(...)` tidak langsung mengeksekusi *future* yang diberikan pada saat itu juga, melainkan hanya memasukkan *task* tersebut ke dalam antrean (task queue) dari *executor*. Setelah *task* dimasukkan ke antrean, alur program pada fungsi `main` (yang bersifat *synchronous*) terus berjalan ke baris selanjutnya, sehingga ia langsung mengeksekusi perintah `println!("Heraldo's Komputer: hey hey");`.

Barulah ketika baris terakhir `executor.run()` dieksekusi, *executor* akan memproses antrean *task* yang ada, menjalankan *future* yang tadi sudah di-*spawn* (mencetak `howdy!`), menunggu *timer* selama 2 detik, lalu mencetak `done!`.

## Experiment 1.3: Multiple Spawn and removing drop

![Screenshot Eksperimen 1.2](docs/images/second-screenshot-without-drop-spawner.png)

![Screenshot Eksperimen 1.2](docs/images/second-screenshot-with-drop-spawner.png)


**Penjelasan:**
* **Multiple Spawn:** Ketika kita melakukan beberapa `spawn`, *tasks* tersebut tidak dijalankan secara berurutan (*sequential*), melainkan secara *concurrent* (bersamaan). Ini terlihat dari semua tulisan `howdy` yang muncul di awal, lalu *timer* 2 detik berjalan bersamaan untuk semua *task*, dan akhirnya semua pesan `done` tercetak hampir di waktu yang sama.
* **Fungsi Spawner:** Bertugas untuk membuat *task* baru (membungkus *future*) dan mengirimkannya ke dalam antrean (*channel* / *queue*) agar nanti bisa dieksekusi.
* **Fungsi Executor:** Bertugas untuk mengambil *task* dari antrean (*ready queue*) dan menjalankannya (memanggil `poll` pada *future* tersebut) hingga selesai.
* **Fungsi Drop(spawner):** Ini adalah kunci kenapa program bisa berhenti. `drop(spawner)` akan menutup interaksi *sender* pada *channel*. Jika *spawner* tidak di-*drop*, *executor* akan terus menyala (menunggu dengan asumsi masih ada *task* yang akan dikirim dari *spawner* lain), sehingga program menjadi *hang* dan tidak mau selesai.