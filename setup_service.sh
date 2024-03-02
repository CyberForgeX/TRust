#!/bin/bash

# Define service name and binary name
SERVICE_NAME="rust_disk_cache"
BINARY_NAME="rust_disk_cache"

# Define paths
SERVICE_DIR="/opt/$SERVICE_NAME"
LOG_DIR="/var/log/$SERVICE_NAME"
CONFIG_DIR="/etc/$SERVICE_NAME"
SYSTEMD_DIR="/etc/systemd/system"

# Create directories
sudo mkdir -p $SERVICE_DIR
sudo mkdir -p $LOG_DIR
sudo mkdir -p $CONFIG_DIR

# Copy binary
sudo cp target/release/$BINARY_NAME $SERVICE_DIR

# Set permissions
sudo chown -R $USER:$USER $SERVICE_DIR
sudo chmod +x $SERVICE_DIR/$BINARY_NAME

# Create configuration file
sudo cp config.json $CONFIG_DIR
sudo chown -R $USER:$USER $CONFIG_DIR

# Create log file
sudo touch $LOG_DIR/$SERVICE_NAME.log
sudo chown -R $USER:$USER $LOG_DIR

# Create systemd service file
sudo tee $SYSTEMD_DIR/$SERVICE_NAME.service > /dev/null <<EOF
[Unit]
Description=Rust Disk Cache Service
After=network.target

[Service]
User=$USER
WorkingDirectory=$SERVICE_DIR
ExecStart=$SERVICE_DIR/$BINARY_NAME
Restart=always
RestartSec=3
Environment="RUST_LOG=info"
StandardOutput=file:$LOG_DIR/$SERVICE_NAME.log
StandardError=file:$LOG_DIR/$SERVICE_NAME.log

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd daemon
sudo systemctl daemon-reload

# Enable and start the service
sudo systemctl enable $SERVICE_NAME
sudo systemctl start $SERVICE_NAME

echo "Service $SERVICE_NAME has been set up and started successfully."
