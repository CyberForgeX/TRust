**🚀 Rust Disk Cache with Encryption 🛡️**

Welcome to the Rust Disk Cache with Encryption repository! 🎉 This project boasts a robust disk caching system implemented in Rust, fortified with encryption to safeguard your precious data. Engineered to optimize performance through asynchronous IO and concurrency, it stands as a stalwart solution adaptable to a myriad of Rust applications. With it you can enhance the memory of your computer with RAM/VRAM using systemctl enable this script. It's not meant as a replacement for hardware, but rather an enhancer. It is highly reccommended to run it on a NVMe M2 because HDD or even SDD will struggle with read and write processes and they are particularly vulnerable to wear and tear of the software. The M2 is also prone to wear and tear, but less so and M2 is very close to real-time data, altough obviosuly it's not as fast as regular hardware type of RAM. It is primarily designed for Linux, but you can adapt the code to work with any OS of your choice with a few tweaks. This program will enhance everything you do on your PC, I mainly build it so it could work with my AI models.

**🌟 Features**

    Asynchronous IO: Harnesses Tokio's asynchronous IO prowess for lightning-fast disk operations, bolstering performance and scalability.
    Concurrency: Fortifies cache operations with thread-safe paradigms using Arc, Mutex, and Semaphore, enabling seamless concurrent access and modification of cache entries.
    Encryption: Ensures data integrity with robust key and nonce management through AES-GCM encryption, guaranteeing impregnable confidentiality.
    Configuration: Flexibly customizable cache behavior with support for loading configuration from environment variables.
    Systemd Service Generation: Empowers effortless deployment of Rust-based services with auto-generation of systemd service files based on user input.

**!! BE AWARE OF WEAR AND TEAR ON THE HARD DRIVE YOUR RUNNING THIS ON!!**
The wear and tear isn't that big on M2 - but it does put a heavier load on it. I haven't tried it on HDD or SDD - so I don't know how that would be. If anyone are interested in trying that out, do so at your own risk. If it went well, could you make an issue for it and mention it?

**💻 Usage**

    Clone the Repository:
   
    git clone https://github.com/your_username/rust-disk-cache.git
   
    cd rust-disk-cache

    cargo run

 Follow the Prompts to create a systemd service file and enable the service or manually configure it do something else.

**🚀 Getting Started**

Integrating the disk cache into your Rust application is a breeze:

    Import Modules: Bring in the DiskCache struct and related modules into your project.
    Create an Instance: Instantiate a new DiskCache with your desired configuration.
    Store and Retrieve Data: Utilize the set method to store data and get method to retrieve data from the cache.
    Customize Behavior: Tailor cache behavior by tweaking configuration and eviction policies.

**🤝 Contributions**

Contributions are heartily welcomed! If you stumble upon any issues or harbor ideas for enhancements, don't hesitate to open an issue or submit a pull request.


**📝 License**

Licensed under the MIT License. See the LICENSE file for details.

**✨ Author**

CyberForgeX

**🙏 Acknowledgments**

Gratitude to the developers of the libraries and frameworks utilized in this project, including Tokio, serde, aes_gcm, and sha2. Their contributions have enriched this endeavor immensely.
