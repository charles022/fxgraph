# **Technical Spec: Zero-Config Local Service Discovery**

Date: November 27, 2025  
Priority: High  
Context: Local Development & Peer-to-Peer Demo Accessibility

## **1\. Objective**

To enable zero-configuration network access to multiple local web services (dashboards, APIs) using dynamic subdomains (e.g., dashboard.hostname.local), while maintaining standard SSH access to the host machine.

## **2\. Architecture Strategy**

We will utilize a "Split-Traffic" architecture on the host machine. We will not bind web services to port 80 directly. Instead, we will use a reverse proxy to multiplex HTTP traffic based on the incoming Host Header, while leaving SSH traffic (Port 22\) untouched.

### **The Stack**

* **mDNS Publisher (Avahi):** Broadcasts multiple CNAME aliases pointing to the host IP.  
* **Reverse Proxy (Caddy):** Listens on Port 80, resolves the Host Header, and routes to internal ports (3000, 8080, etc.).  
* **Application Layer:** Existing Rust/Python services running on non-privileged ports (localhost only).

### **Traffic Flow**

1. **SSH Request:** ssh user@host.local → Port 22 (Handled natively by OS).  
2. **Web Request A:** http://dash.host.local → Port 80 → Caddy → Localhost:3000.  
3. **Web Request B:** http://admin.host.local → Port 80 → Caddy → Localhost:8080.

## **3\. Implementation Requirements**

### **A. Dynamic mDNS Broadcasting**

We need a background process that detects the machine's current hostname and broadcasts the required subdomains.

* *Constraint:* Must handle dynamic hostnames (cannot hardcode "mymachine").  
* *Tool:* avahi-publish

**Required Script Logic (publish\_services.sh):**

\#\!/bin/bash  
HOST=$(hostname)  
IP=$(hostname \-I | awk '{print $1}')  
\# List of subdomains to broadcast  
SERVICES=("dashboard" "analytics" "admin")

for SVC in "${SERVICES\[@\]}"; do  
    avahi-publish \-a \-R "$SVC.$HOST.local" "$IP" &  
done  
wait

### **B. Reverse Proxy Configuration**

We will use Caddy for its environment variable support and automatic HTTPS disablement for .local domains.

**Required Caddyfile Config:**

\# Global: Bind to HTTP only (no auto-https for .local to avoid cert errors)  
{  
    auto\_https off  
}

\# Service 1: Main Dashboard  
http://dashboard.{$HOSTNAME}.local {  
    reverse\_proxy localhost:3000  
}

\# Service 2: Secondary App  
http://analytics.{$HOSTNAME}.local {  
    reverse\_proxy localhost:8080  
}

\# Root Fallback  
http://{$HOSTNAME}.local {  
    respond "Gateway Online. Services available at dashboard.{$HOSTNAME}.local"  
}

### **C. System Integration (Systemd)**

The mDNS broadcaster and Caddy must start automatically on boot.

1. **Firewall:** Open TCP Port 80 permanently.  
2. **Service:** Create a systemd unit (local-gateway.service) that executes the publish\_services.sh script and starts Caddy.

## **4\. Success Criteria**

1. **SSH is Unaffected:** ssh user@hostname.local connects successfully.  
2. **Service Discovery:** A different device on the WiFi can ping dashboard.hostname.local.  
3. **Routing:** Navigating to dashboard.hostname.local in a browser loads the app running on port 3000\.  
4. **Zero-Config:** The setup works immediately after a reboot without manual intervention.

**Next Actions for Engineer:**

1. Install caddy and avahi-tools.  
2. Deploy the Caddyfile to /etc/caddy/Caddyfile.  
3. Implement the broadcast script as a systemd service.