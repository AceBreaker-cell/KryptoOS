#!/bin/bash
#
# KryptonOS Installation Script
# Installs KryptonOS as a system service with proper permissions and capabilities
#

set -euo pipefail

# Configuration
INSTALL_DIR="/opt/kryptonos"
BINARY_NAME="kryptonos"
SERVICE_NAME="kryptonos"
USER_NAME="kryptonos"
GROUP_NAME="kryptonos"
LOG_DIR="/var/log/kryptonos"
DATA_DIR="/var/lib/kryptonos"
RUN_DIR="/var/run/kryptonos"
CONFIG_DIR="/etc/kryptonos"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
log_info() { echo -e "${GREEN}[INFO]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    log_error "This script must be run as root"
    exit 1
fi

# Detect architecture and OS
log_info "Detecting system architecture..."
ARCH=$(uname -m)
OS=$(uname -s)

log_info "Architecture: $ARCH"
log_info "OS: $OS"

# Only support Linux for now
if [[ "$OS" != "Linux" ]]; then
    log_error "This installer only supports Linux"
    exit 1
fi

# Create system user and group if they don't exist
log_info "Creating system user and group..."
if ! getent group "$GROUP_NAME" > /dev/null 2>&1; then
    groupadd --system "$GROUP_NAME"
    log_info "Created group: $GROUP_NAME"
fi

if ! id "$USER_NAME" &>/dev/null; then
    useradd --system --gid "$GROUP_NAME" --no-create-home --shell /usr/sbin/nologin "$USER_NAME"
    log_info "Created user: $USER_NAME"
fi

# Create directory structure
log_info "Creating directory structure..."
mkdir -p "$INSTALL_DIR"
mkdir -p "$LOG_DIR"
mkdir -p "$DATA_DIR"
mkdir -p "$RUN_DIR"
mkdir -p "$CONFIG_DIR"

# Set ownership
chown -R "$USER_NAME:$GROUP_NAME" "$INSTALL_DIR"
chown -R "$USER_NAME:$GROUP_NAME" "$LOG_DIR"
chown -R "$USER_NAME:$GROUP_NAME" "$DATA_DIR"
chown -R "$USER_NAME:$GROUP_NAME" "$RUN_DIR"
chown -R "$USER_NAME:$GROUP_NAME" "$CONFIG_DIR"

# Set permissions
chmod 750 "$INSTALL_DIR"
chmod 750 "$LOG_DIR"
chmod 750 "$DATA_DIR"
chmod 750 "$RUN_DIR"
chmod 750 "$CONFIG_DIR"

# Copy the binary (assuming it's already built)
log_info "Installing binary..."
if [[ ! -f "target/release/$BINARY_NAME" ]]; then
    log_error "Binary not found at target/release/$BINARY_NAME"
    log_info "Please build the application first with: cargo build --release"
    exit 1
fi

cp "target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
chown "$USER_NAME:$GROUP_NAME" "$INSTALL_DIR/$BINARY_NAME"
chmod 750 "$INSTALL_DIR/$BINARY_NAME"

# Set Linux capabilities for network operations
log_info "Setting Linux capabilities..."
setcap cap_net_admin,cap_net_raw,cap_sys_ptrace,cap_dac_override+ep "$INSTALL_DIR/$BINARY_NAME"
log_info "Capabilities set: cap_net_admin, cap_net_raw, cap_sys_ptrace, cap_dac_override"

# Create systemd service file
log_info "Creating systemd service..."
cat > "/etc/systemd/system/$SERVICE_NAME.service" << EOF
[Unit]
Description=KryptonOS Zero-Trust Host Access Controller
After=network.target auditd.service
Wants=network.target

[Service]
Type=notify
User=$USER_NAME
Group=$GROUP_NAME
ExecStart=$INSTALL_DIR/$BINARY_NAME
PIDFile=$RUN_DIR/$SERVICE_NAME.pid
Restart=on-failure
RestartSec=5
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
NoNewPrivileges=yes
CapabilityBoundingSet=CAP_NET_ADMIN CAP_NET_RAW CAP_SYS_PTRACE CAP_DAC_OVERRIDE
AmbientCapabilities=CAP_NET_ADMIN CAP_NET_RAW CAP_SYS_PTRACE CAP_DAC_OVERRIDE
SystemCallFilter=@system-service
SystemCallError=EPERM
Nice=0
IOSchedulingClass=2
IOSchedulingPriority=0
CPUSchedulingPolicy=other
CPUSchedulingPriority=0
MemoryLimit=50M
CPUQuota=20%
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# Set permissions on service file
chmod 644 "/etc/systemd/system/$SERVICE_NAME.service"
chown root:root "/etc/systemd/system/$SERVICE_NAME.service"

# Reload systemd and enable service
log_info "Reloading systemd daemon..."
systemctl daemon-reload

log_info "Enabling service to start on boot..."
systemctl enable "$SERVICE_NAME.service"

# Create desktop entry for GUI applications
log_info "Creating desktop entry..."
# Note: In our implementation, the logo is embedded in the binary, so we use a generic icon
# or we could extract it at runtime, but for the desktop file we'll use a placeholder
cat > "/usr/share/applications/kryptonos.desktop" << EOF
[Desktop Entry]
Name=KryptonOS
Comment=Zero-Trust Host Access Controller
Exec=$INSTALL_DIR/$BINARY_NAME
Icon=utilities-terminal-shield  # Using a generic security-themed icon
Terminal=false
Type=Application
Categories=System;Security;
StartupNotify=true
EOF

# Start the service
log_info "Starting KryptonOS service..."
systemctl start "$SERVICE_NAME.service"

# Wait a moment and check status
sleep 2
if systemctl is-active --quiet "$SERVICE_NAME.service"; then
    log_info "KryptonOS has been successfully installed and started!"
    log_info "Service status: $(systemctl show -p ActiveState --value $SERVICE_NAME)"
    log_info "You can manage the service with:"
    log_info "  sudo systemctl status $SERVICE_NAME"
    log_info "  sudo journalctl -u $SERVICE_NAME -f"
else
    log_error "Failed to start KryptonOS service"
    log_info "Checking service status:"
    systemctl status "$SERVICE_NAME.service" --no-pager
    log_info "Checking logs:"
    journalctl -u "$SERVICE_NAME.service" --no-pager -n 20
    exit 1
fi

log_info "Installation complete!"
log_info "Binary location: $INSTALL_DIR/$BINARY_NAME"
log_info "Service name: $SERVICE_NAME"
log_info "Log directory: $LOG_DIR"
log_info "Data directory: $DATA_DIR"