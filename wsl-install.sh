sudo apt update;
sudo apt upgrade;
sudo apt install -y nano vim unzip wget git git-lfs openssh-client apt-transport-https wget \
  software-properties-common build-essential libssl-dev gcc g++ gdb ninja-build doxygen graphviz \
  googletest protobuf-compiler make cmake libssl-dev curl

PATH="/root/.cargo/bin:${PATH}"
wget https://packages.microsoft.com/config/ubuntu/21.04/packages-microsoft-prod.deb -O packages-microsoft-prod.deb;
sudo dpkg -i packages-microsoft-prod.deb
sudo apt update;
sudo apt install powershell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

