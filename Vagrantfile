# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure("2") do |config|
  config.vm.box = "monsenso/macos-10.13"

  config.vm.synced_folder ".", "/vagrant", type: "rsync", owner: "vagrant", group: "staff"

  config.vm.provider "virtualbox" do |vb|
    vb.gui = true
  end
end
