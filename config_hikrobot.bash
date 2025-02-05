sudo apt install wget

wget https://github.com/shioko-chan/quasar_trajectory/releases/download/hikrobotSDK/MVS-3.0.1_aarch64_20241128.deb -O MVS-3.0.1_aarch64_20241128.deb


sudo apt install ./MVS-3.0.1_aarch64_20241128.deb

cat >> ~/.bashrc <<EOF
export LD_LIBRARY_PATH=/opt/MVS/bin:$LD_LIBRARY_PATH
export PATH=/opt/MVS/bin:$PATH
EOF