wget http://fishros.com/install -O fishros && . fishros

sudo apt update
sudo apt upgrade

sudo apt install figlet fortune-mod lolcat curl 

export RUSTUP_DIST_SERVER=https://mirrors.tuna.tsinghua.edu.cn/rustup
export RUSTUP_UPDATE_ROOT=https://mirrors.tuna.tsinghua.edu.cn/rustup/rustup

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

cat > ~/.cargo/config.toml <<EOF
[source.crates-io]
replace-with = "ustc"

[source.ustc]
registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"
EOF

cat >> ~/.bashrc <<EOF
# >>> An Auspicious Script
echo "You are coding in: " | lolcat
figlet "Orin Nano" | lolcat
echo "As the adage wisely states: " | lolcat 
figlet "Exitus Acta Probat." | lolcat
echo "Here is your fortune for reflection: " | lolcat
echo
fortune | lolcat
# <<<
EOF