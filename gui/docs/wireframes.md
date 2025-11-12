# CrypRQ Desktop GUI Wireframes

## Design Principles

- **Minimalistic & Professional**: Clean, flat UI with clear visual hierarchy
- **Security-First**: UI reflects trust and technical sophistication
- **Accessible**: High contrast, readable fonts, clear status indicators
- **Responsive**: Adapts to different window sizes and high DPI displays

## Color Scheme

### Light Theme
- Background: `#FFFFFF`
- Surface: `#F5F5F5`
- Primary: `#1DE9B6` (Teal)
- Text: `#212121`
- Text Secondary: `#757575`
- Success: `#4CAF50` (Green)
- Error: `#F44336` (Red)
- Warning: `#FF9800` (Orange)

### Dark Theme
- Background: `#121212`
- Surface: `#1E1E1E`
- Primary: `#1DE9B6` (Teal)
- Text: `#E0E0E0`
- Text Secondary: `#B0B0B0`
- Success: `#66BB6A` (Green)
- Error: `#EF5350` (Red)
- Warning: `#FFB74D` (Orange)

## Layout Structure

```

  Sidebar (240px)    Main Content Area (flex)         
                                                        
  CrypRQ              [Page Content]                   
  Post-quantum VPN                                      
                                                        
                                        
                                                        
   Dashboard                                          
   Peers                                              
    Settings                                          
                                                        

```

## Screen 1: Dashboard

### Header
- Title: "Dashboard"
- Connection Status Card (prominent)

### Connection Status Card
```

  Connection Status                          [Connect]
   Connected                                         
                                                      
  Peer ID: 12D3KooW...                               
  Next Rotation: 4:32                                
  Bytes In: 1.23 MB    Bytes Out: 456 KB             

```

- Large status indicator (green dot = connected, red = disconnected)
- Connect/Disconnect button (primary action)
- Peer ID (truncated, monospace font)
- Rotation timer (countdown, teal highlight)
- Throughput metrics (in/out)

### Recent Activity Section
```

  Recent Activity                                     
   
  1415 [INFO] Connected to peer 12D3KooW...     
  1410 [INFO] Key rotation completed             
  1405 [INFO] Handshake successful               
  1400 [INFO] Starting listener on /ip4/...      

```

- Scrollable log view (last 10-20 entries)
- Color-coded log levels
- Timestamp + level + message
- Monospace font for technical details

## Screen 2: Peers Management

### Header
- Title: "Peers"
- "+ Add Peer" button (primary)

### Peer List
```

   12D3KooW...ABC                    [Connect] [Remove]
     /ip4/192.168.1.100/udp/9999/quic-v1             
     Last seen: 2024-01-15 1400                  
                                                      
   12D3KooW...XYZ                    [Connect] [Remove]
     /ip4/10.0.0.5/udp/9999/quic-v1                  
     Last seen: Never                                 

```

- Status dot ( = connected,  = disconnected)
- Peer ID (truncated, monospace)
- Multiaddr (full, monospace, secondary color)
- Last seen timestamp
- Connect button (enabled when disconnected)
- Remove button (with confirmation dialog)

### Add Peer Dialog (Modal)
```

  Add Peer                                    [X]     
                                                      
  Peer ID                                             
  [12D3KooW...________________]                       
                                                      
  Multiaddr                                           
  [/ip4/127.0.0.1/udp/9999/quic-v1________]          
                                                      
                              [Cancel]  [Add]        

```

- Input fields for Peer ID and Multiaddr
- Validation feedback
- Cancel/Add buttons

## Screen 3: Settings

### Key Rotation Section
```

  Key Rotation                                        
   
  Rotation Interval (seconds)                        
  [300________________]                              
  Current: 5 minutes                                 

```

- Number input (60-3600 seconds)
- Live preview of minutes
- Tooltip explaining rotation security

### Logging Section
```

  Logging                                            
   
  Log Level                                          
  [Info ]                                           
    • Error                                          
    • Warning                                        
    • Info                                           
    • Debug                                          

```

- Dropdown for log level
- Description of each level

### Transport Section
```

  Transport                                          
   
  Multiaddr                                          
  [/ip4/0.0.0.0/udp/9999/quic-v1________________]   
                                                      
  UDP Port                                           
  [9999________]                                      

```

- Multiaddr input (full width, monospace)
- UDP port input (number, 1024-65535)
- Validation and error messages

### Appearance Section
```

  Appearance                                         
   
  Theme                                              
  [System ]                                         
    • Light                                          
    • Dark                                           
    • System                                         

```

- Theme selector (Light/Dark/System)
- System option respects OS preference

## System Tray Menu

```

 Show CrypRQ      
  
 Connect          
 Disconnect       
  
 Quit             

```

- Quick access to main actions
- Status indicator in tray icon
- Right-click context menu

## Status Indicators

### Connection Status
-  **Green dot**: Connected
-  **Red dot**: Disconnected
-  **Yellow dot**: Connecting/Rotating

### Peer Status
-  **Filled circle**: Connected
-  **Empty circle**: Disconnected
- ⏳ **Spinner**: Connecting

## Typography

- **Headings**: System font, 600 weight
- **Body**: System font, 400 weight
- **Monospace**: For peer IDs, multiaddrs, logs (source-code-pro, Menlo, Consolas)

## Responsive Behavior

- **Minimum width**: 800px
- **Sidebar**: Fixed 240px width
- **Content**: Flexible, max-width 1200px with padding
- **High DPI**: 2x scaling for icons/assets

## Accessibility

- Keyboard navigation (Tab, Enter, Escape)
- Screen reader labels
- High contrast mode support
- Focus indicators on interactive elements
- Tooltips for technical terms

