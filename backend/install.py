#!/usr/bin/env python3
"""Geführte Installation von Totenrunde-Web mit Docker, Certbot und HTTP/3."""

import argparse
import os
from pathlib import Path
import re
import shlex
import shutil
import socket
import subprocess
import sys
import time
import urllib.request

BASE_DIR = Path(__file__).resolve().parent
COMPOSE_FILE = BASE_DIR / "compose.yaml"
ENV_FILE = BASE_DIR / ".env"
CERT_DIR = BASE_DIR / "certs"
DEPLOY_SCRIPT = BASE_DIR / "certbot-deploy.sh"
HOOK_FILE = Path("/etc/letsencrypt/renewal-hooks/deploy/totenrunde.sh")
DOCKER_REPO_FILE = Path("/etc/yum.repos.d/docker-ce.repo")
DOCKER_REPO_URL = "https://download.docker.com/linux/centos/docker-ce.repo"


class SetupError(RuntimeError):
    pass


class Installer:
    def __init__(self, dry_run: bool):
        self.dry_run = dry_run

    def run(self, command, *, env=None, check=True, capture=False):
        print("+ " + shlex.join(str(part) for part in command))
        if self.dry_run:
            return subprocess.CompletedProcess(command, 0, stdout="", stderr="")
        return subprocess.run(
            command,
            env=env,
            check=check,
            text=True,
            capture_output=capture,
        )

    def write(self, path: Path, content: str, mode: int):
        print(f"+ Datei {path} schreiben (Modus {mode:04o})")
        if self.dry_run:
            return
        path.parent.mkdir(parents=True, exist_ok=True)
        temporary = path.with_name(path.name + ".tmp")
        temporary.write_text(content, encoding="utf-8")
        temporary.chmod(mode)
        os.replace(temporary, path)

    def compose(self, *arguments, capture=False, check=True):
        return self.run(
            [
                "docker",
                "compose",
                "--env-file",
                str(ENV_FILE),
                "-f",
                str(COMPOSE_FILE),
                *arguments,
            ],
            capture=capture,
            check=check,
        )


def parse_arguments():
    parser = argparse.ArgumentParser(
        description=(
            "Richtet Totenrunde-Web auf Ubuntu 24.04+ oder Rocky Linux 10+ "
            "mit einem echten TLS-Zertifikat und ausschließlich HTTP/3 ein."
        )
    )
    parser.add_argument("--domain", help="Öffentlicher DNS-Name, z. B. spiel.example.de")
    parser.add_argument("--email", help="E-Mail-Adresse für Let's-Encrypt-Hinweise")
    parser.add_argument("--port", type=int, help="Öffentlicher UDP-Port (Standard: 443)")
    parser.add_argument(
        "--ssh-port",
        type=int,
        default=22,
        help="SSH-Port, der in der Firewall offen bleibt (Standard: 22)",
    )
    parser.add_argument(
        "--non-interactive",
        action="store_true",
        help="Keine Rückfragen; Domain und E-Mail sind erforderlich",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Nur Schritte anzeigen, nichts installieren oder verändern",
    )
    return parser.parse_args()


def read_env():
    values = {}
    if not ENV_FILE.exists():
        return values
    for line in ENV_FILE.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        values[key] = value
    return values


def ask(label, current=None, secret_default=None):
    suffix = f" [{current}]" if current else ""
    while True:
        value = input(f"{label}{suffix}: ").strip()
        if value:
            return value
        if current:
            return current
        if secret_default:
            return secret_default


def validate_domain(value):
    value = value.strip().rstrip(".").lower().encode("idna").decode("ascii")
    labels = value.split(".")
    valid = (
        len(value) <= 253
        and len(labels) >= 2
        and all(
            0 < len(label) <= 63
            and re.fullmatch(r"[a-z0-9](?:[a-z0-9-]*[a-z0-9])?", label)
            for label in labels
        )
    )
    if not valid:
        raise SetupError(
            "Bitte einen gültigen öffentlichen Domainnamen ohne https:// oder Pfad angeben."
        )
    return value


def validate_email(value):
    value = value.strip()
    if not re.fullmatch(r"[^\s@]+@[^\s@]+\.[^\s@]+", value):
        raise SetupError("Bitte eine gültige E-Mail-Adresse angeben.")
    return value


def gather_settings(args):
    existing = read_env() if not args.dry_run else {}
    domain = args.domain or existing.get("DOMAIN")
    email = args.email or existing.get("CERTBOT_EMAIL")
    if domain == "spiel.example.de":
        domain = None
    if email == "admin@example.de":
        email = None

    if args.dry_run:
        domain = domain or "spiel.example.de"
        email = email or "admin@example.de"
    elif not args.non_interactive:
        domain = ask("Öffentlicher Domainname", domain)
        email = ask("E-Mail für Let's Encrypt", email)

    if not domain or not email:
        raise SetupError(
            "Bei --non-interactive müssen --domain und --email angegeben werden "
            "(oder bereits in .env stehen)."
        )

    port = args.port or int(existing.get("PORT", "443"))
    if not 1 <= port <= 65535 or not 1 <= args.ssh_port <= 65535:
        raise SetupError("Die Ports müssen zwischen 1 und 65535 liegen.")
    return {
        "domain": validate_domain(domain),
        "email": validate_email(email),
        "port": port,
        "ssh_port": args.ssh_port,
        "image": existing.get("IMAGE", "ghcr.io/zerox80/totenrunde-web"),
        "tag": existing.get("TAG", "latest"),
    }


def supported_system():
    values = {}
    for line in Path("/etc/os-release").read_text(encoding="utf-8").splitlines():
        if "=" in line:
            key, value = line.split("=", 1)
            values[key] = value.strip().strip('"')
    system = values.get("ID", "")
    try:
        major = int(values.get("VERSION_ID", "0").split(".")[0])
    except ValueError as error:
        raise SetupError("Die Betriebssystemversion konnte nicht erkannt werden.") from error
    if system == "ubuntu" and major >= 24:
        return "ubuntu"
    if system == "rocky" and major >= 10:
        return "rocky"
    raise SetupError(
        f"{values.get('PRETTY_NAME', system)} wird nicht unterstützt. "
        "Benötigt wird Ubuntu 24.04+ oder Rocky Linux 10+."
    )


def elevate_if_needed():
    if os.name != "posix":
        raise SetupError("Dieser Installer unterstützt ausschließlich Linux.")
    if os.geteuid() == 0:
        return
    if not shutil.which("sudo"):
        raise SetupError("Bitte das Skript als root ausführen; sudo wurde nicht gefunden.")
    print("Für die Systemeinrichtung sind root-Rechte erforderlich; sudo wird gestartet.")
    os.execvp("sudo", ["sudo", sys.executable, str(Path(__file__).resolve()), *sys.argv[1:]])


def install_packages(installer, system):
    missing_docker = not shutil.which("docker")
    missing_compose = not shutil.which("docker") or installer.run(
        ["docker", "compose", "version"], check=False, capture=True
    ).returncode != 0
    missing_certbot = not shutil.which("certbot")
    firewall_command = "ufw" if system == "ubuntu" else "firewall-cmd"
    missing_firewall = not shutil.which(firewall_command)
    if not any((missing_docker, missing_compose, missing_certbot, missing_firewall)):
        print("Docker, Compose, Certbot und die Firewall sind bereits vorhanden.")
    elif system == "ubuntu":
        packages = []
        if missing_docker:
            packages.append("docker.io")
        if missing_compose:
            packages.append("docker-compose-v2")
        if missing_certbot:
            packages.append("certbot")
        if missing_firewall:
            packages.append("ufw")
        installer.run(["apt-get", "update"])
        installer.run(["apt-get", "install", "-y", *packages])
    else:
        installer.run(["dnf", "install", "-y", "epel-release"])
        if missing_docker or missing_compose:
            if not DOCKER_REPO_FILE.exists() and not installer.dry_run:
                print(f"+ Docker-Repository von {DOCKER_REPO_URL} laden")
                with urllib.request.urlopen(DOCKER_REPO_URL, timeout=30) as response:
                    installer.write(DOCKER_REPO_FILE, response.read().decode("utf-8"), 0o644)
            packages = ["docker-ce", "docker-ce-cli", "containerd.io"]
            if missing_compose:
                packages.append("docker-compose-plugin")
            installer.run(["dnf", "install", "-y", *packages])
        if missing_certbot:
            installer.run(["dnf", "install", "-y", "certbot"])
        if missing_firewall:
            installer.run(["dnf", "install", "-y", "firewalld"])

    installer.run(["systemctl", "enable", "--now", "docker"])
    installer.run(["docker", "compose", "version"])


def configure_firewall(installer, system, port, ssh_port):
    if system == "ubuntu":
        installer.run(["ufw", "allow", f"{ssh_port}/tcp", "comment", "SSH"])
        installer.run(["ufw", "allow", "80/tcp", "comment", "Certbot"])
        installer.run(["ufw", "allow", f"{port}/udp", "comment", "Totenrunde HTTP3"])
        installer.run(["ufw", "--force", "enable"])
        installer.run(["ufw", "status", "verbose"])
    else:
        installer.run(["systemctl", "enable", "--now", "firewalld"])
        installer.run(["firewall-cmd", "--permanent", f"--add-port={ssh_port}/tcp"])
        installer.run(["firewall-cmd", "--permanent", "--add-port=80/tcp"])
        installer.run(["firewall-cmd", "--permanent", f"--add-port={port}/udp"])
        installer.run(["firewall-cmd", "--reload"])
    print(
        "Router und Cloud-Firewall müssen zusätzlich TCP/80 sowie "
        f"UDP/{port} zu diesem Server weiterleiten."
    )


def check_dns_and_challenge_port(domain):
    try:
        addresses = sorted(
            {item[4][0] for item in socket.getaddrinfo(domain, 443, type=socket.SOCK_DGRAM)}
        )
    except socket.gaierror as error:
        raise SetupError(
            f"Der Domainname {domain} kann nicht aufgelöst werden. Bitte zuerst den DNS-Eintrag setzen."
        ) from error
    print(f"DNS für {domain}: {', '.join(addresses)}")
    print("Stelle sicher, dass dieser Server über die öffentliche Domain per TCP/80 erreichbar ist.")
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
            listener.bind(("0.0.0.0", 80))
    except OSError as error:
        raise SetupError(
            "TCP/80 ist belegt. Certbot Standalone benötigt diesen Port kurzzeitig für die "
            "Let's-Encrypt-Prüfung; ein vorhandener Webserver muss dafür pausieren."
        ) from error


def write_configuration(installer, settings):
    content = (
        f"PORT={settings['port']}\n"
        f"IMAGE={settings['image']}\n"
        f"TAG={settings['tag']}\n"
        f"DOMAIN={settings['domain']}\n"
        f"TLS_SERVER_NAME={settings['domain']}\n"
        "CERT_DIR=./certs\n"
        f"CERTBOT_EMAIL={settings['email']}\n"
    )
    installer.write(ENV_FILE, content, 0o600)
    installer.run(["chmod", "+x", str(DEPLOY_SCRIPT)])


def obtain_certificate(installer, settings):
    installer.run(
        [
            "certbot",
            "certonly",
            "--standalone",
            "--preferred-challenges",
            "http",
            "--http-01-port",
            "80",
            "--domain",
            settings["domain"],
            "--email",
            settings["email"],
            "--agree-tos",
            "--non-interactive",
            "--keep-until-expiring",
        ]
    )
    lineage = f"/etc/letsencrypt/live/{settings['domain']}"
    environment = os.environ.copy()
    environment.update(
        CERT_DIR=str(CERT_DIR), RENEWED_LINEAGE=lineage, SKIP_RESTART="1"
    )
    installer.run([str(DEPLOY_SCRIPT)], env=environment)


def install_renewal_hook(installer, domain):
    content = f"""#!/bin/sh
set -eu
if [ "${{RENEWED_LINEAGE:-}}" = {shlex.quote(f'/etc/letsencrypt/live/{domain}')} ]; then
    export CERT_DIR={shlex.quote(str(CERT_DIR))}
    export COMPOSE_FILE={shlex.quote(str(COMPOSE_FILE))}
    export ENV_FILE={shlex.quote(str(ENV_FILE))}
    exec {shlex.quote(str(DEPLOY_SCRIPT))}
fi
"""
    installer.write(HOOK_FILE, content, 0o755)
    for unit in ("certbot.timer", "certbot-renew.timer"):
        result = installer.run(["systemctl", "cat", unit], check=False, capture=True)
        if result.returncode == 0 or installer.dry_run:
            installer.run(["systemctl", "enable", "--now", unit])
            return
    raise SetupError("Kein Certbot-Erneuerungs-Timer gefunden; automatische Erneuerung fehlt.")


def start_and_verify(installer):
    installer.compose("pull")
    installer.compose("up", "-d")
    installer.compose("ps")
    if installer.dry_run:
        return

    deadline = time.monotonic() + 90
    while time.monotonic() < deadline:
        container = installer.compose("ps", "-q", "web", capture=True).stdout.strip()
        if container:
            result = installer.run(
                [
                    "docker",
                    "inspect",
                    "--format",
                    "{{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}",
                    container,
                ],
                capture=True,
            )
            status = result.stdout.strip()
            if status == "healthy":
                print("Der HTTP/3-Healthcheck ist erfolgreich.")
                return
            if status == "unhealthy":
                break
        time.sleep(3)
    installer.compose("logs", "--tail=100", "web", check=False)
    raise SetupError("Der Container wurde nicht fehlerfrei gestartet; siehe Logs oben.")


def main():
    args = parse_arguments()
    installer = Installer(args.dry_run)
    try:
        settings = gather_settings(args)
        print(
            "\nTotenrunde-Einrichtung:\n"
            f"  Adresse: https://{settings['domain']}:{settings['port']}\n"
            "  Protokoll: ausschließlich HTTP/3 (QUIC/UDP)\n"
            "  Zertifikat: Let's Encrypt über Certbot\n"
        )
        if not args.dry_run and not args.non_interactive:
            if input("Installation jetzt ausführen? [j/N] ").strip().lower() not in ("j", "ja"):
                print("Abgebrochen; es wurden keine Änderungen vorgenommen.")
                return 0
        if not args.dry_run:
            system = supported_system()
            elevate_if_needed()
            check_dns_and_challenge_port(settings["domain"])
        else:
            system = "ubuntu"
            print("DRY-RUN: Es werden keine Befehle ausgeführt und keine Dateien verändert.")
        install_packages(installer, system)
        configure_firewall(installer, system, settings["port"], settings["ssh_port"])
        write_configuration(installer, settings)
        obtain_certificate(installer, settings)
        install_renewal_hook(installer, settings["domain"])
        start_and_verify(installer)
        print(
            f"\nFertig: https://{settings['domain']}:{settings['port']}\n"
            "Einen aktuellen Desktop-Browser mit HTTP/3- und WebGL-2-Unterstützung verwenden."
        )
        return 0
    except (SetupError, subprocess.CalledProcessError, OSError, ValueError) as error:
        print(f"\nFehler: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
