# PMD Installation Guide

To run `pmd_runner`, you need to have **PMD** installed on your system.

Please follow the installation steps below to download and set up PMD correctly.
Make sure that the `pmd` command is available in your system's PATH so that
`pmd_runner` can invoke it successfully.

## Step 1: Download PMD

```bash
wget https://github.com/pmd/pmd/releases/download/pmd_releases/7.13.0/pmd-bin-7.13.0.zip
```

## Step 2: Extract the Archive

```bash
unzip pmd-bin-7.13.0.zip -d ~/pmd
```

## Step 3: Add PMD to PATH (optional)

```bash
echo 'export PATH=$PATH:~/pmd/pmd-bin-7.13.0/bin' >> ~/.bashrc
source ~/.bashrc
```

## Step 4: Verify Installation

```bash
pmd -version
```
