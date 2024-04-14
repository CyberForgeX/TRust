<div align="center">
  <img src="https://github.com/CyberForgeX/TRust/assets/22517914/8792837f-0e8f-4e6e-9b82-f00444b69a02" alt="TRust Logo" height="200" width="300">
</div>
<h2 align="center">TRust - Total Resource Utilization System Tool</h2>

Welcome to **TRust**, where cutting-edge hard drive caching meets encryption to turbocharge your data access and safeguard your information. Designed with Rust's prowess, TRust introduces an innovative caching layer using hard drive as a scalable robust computational speed for all types of hardware, optimized for asynchronous IO and concurrency, it's a game-changer for enhancing your computing experience without replacing hardware. It shines on NVMe M2 drives, offering near-real-time data access speeds, and is adaptable across platforms with some tweaks. Dive into a solution that not only accelerates your applications but also wraps your data in robust security.

TRust is a cutting-edge performance enhancement tool designed to optimize your entire system's hardware resources using advanced Rust programming techniques. It provides a comprehensive solution for managing and accelerating the performance of your CPU, GPU, RAM, and M.2 drives. From asynchronous disk operations to smart caching strategies, TRust ensures that your hardware operates at its peak efficiency

<div align="center">
<a href="https://www.buymeacoffee.com/CyberForgeX" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/v2/arial-yellow.png" alt="Buy Me A Coffee" style="height: 60px !important;width: 217px !important;" ></a>
</div>

## 🌠 Key Features


### Asynchronous Disk IO
- **Embrace Tokio**: Utilize Tokio's powerful asynchronous runtime for non-blocking file operations, dramatically enhancing the efficiency of IO-bound tasks on your M.2 drives and other storage devices.

### Smart LRU Caching
- **Dynamic Data Prioritization**: Implement an intelligent Least Recently Used (LRU) caching mechanism that automatically prioritizes the most frequently accessed data, significantly speeding up both read and write operations on your system.

### Built-In Encryption
- **Secure Your Data**: Enhance data security with built-in encryption for your cached data, ensuring that your sensitive information is protected without compromising on system performance.

### Systemd Integration
- **Seamless Deployment**: Deploy TRust effortlessly with full systemd support, including automated service file generation for Linux systems, facilitating easy management and startup.

### Interactive CLI
- **User-Friendly Management**: Configure and control your system's caching settings through a robust command-line interface that offers simplicity and flexibility for both novice and advanced users.

### Flexible and Configurable
- **Customizable Performance**: Tailor TRust to fit your specific needs, allowing you to adjust cache size, storage locations, and other parameters to optimize performance based on your hardware setup.

### Concurrency-Ready
- **Safe Concurrent Access**: Leverage Rust’s Arc and Mutex to manage concurrent access to resources safely, ensuring data integrity across multiple threads and processes.

### Full Hardware Integration
- **CPU and GPU Optimization**: Utilize TRust to optimize processing tasks across CPU and GPU, improving computation efficiency and application performance.
- **RAM Efficiency**: Enhance memory management with intelligent algorithms that ensure optimal RAM utilization, reducing unnecessary paging and boosting response times.


## 🚀 Elevate Your Performance

TRust is not just about speeding up disk access; it's about transforming your computer's responsiveness. Whether for web servers, AI model inferences, database operations, or even gaming servers, TRust offers a versatile solution that scales with your needs.

### Streamlining Web Services

Cache API responses or session data to slash response times and reduce database load, making user experiences snappier.

### Accelerating AI and Machine Learning

Store model predictions or features in cache to minimize inference times, enabling faster decision-making.

### Optimizing Database Interactions

Implement query result caching to alleviate database stress, ensuring quick data retrieval for repetitive queries.

### Enhancing Gaming Experiences

Maintain game state and leaderboard information in cache for real-time updates, keeping players engaged.

## 🛠 Getting Started

Setting up TRust is straightforward:

1. **Clone the repository**:
   ```bash
   git clone https://github.com/CyberForgeX/rust-disk-cache.git
   cd rust-disk-cache
   ```

2. **Run with Cargo**:
   ```bash
   cargo run
   ```
   
   Follow the interactive CLI to configure your caching solution and systemd service.

## 💡 Contribute to Innovation

Join us in refining TRust! Whether it's through issue reporting, code contributions, or feature suggestions, your input is invaluable. Dive into the code, tweak, test, and share your experiences. Together, we can push the boundaries of what's possible with caching and encryption.

## 📜 License

TRust is proudly open source, licensed under the MIT License. Dive into the project, use it, modify it, and distribute it according to the license terms.

## 🌟 Acknowledgments

A heartfelt thank you to everyone who uses, contributes to, and supports TRust. Your engagement fuels the journey toward making computing faster and data security stronger.

