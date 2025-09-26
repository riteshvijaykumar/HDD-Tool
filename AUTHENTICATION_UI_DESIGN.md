# 🎨 Authentication UI Screenshots & Design

## 📱 **Login Page Design**

```
┌─────────────────────────────────────────────────────────┐
│                                                         │
│                🛡️ SHREDX Authentication                 │
│                                                         │
│        ┌─────────────────────────────────────────┐      │
│        │                                         │      │
│        │          Login to Continue              │      │
│        │                                         │      │
│        │       👤 Username: [admin         ] 👁  │      │
│        │                                         │      │
│        │       🔒 Password: [••••••••••••] 👁   │      │
│        │                                         │      │
│        │         🚀 [    Login    ]              │      │
│        │                                         │      │
│        │    Need to create users? Create User    │      │
│        │                                         │      │
│        │          Default: admin / admin123      │      │
│        │                                         │      │
│        └─────────────────────────────────────────┘      │
│                                                         │
│              ✅ Welcome back, admin!                    │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

## 👥 **Create User Page Design**

```
┌─────────────────────────────────────────────────────────┐
│                                                         │
│              👥 Create New User                         │
│                                                         │
│        ┌─────────────────────────────────────────┐      │
│        │                                         │      │
│        │  👤 Username:  [john_doe            ]  │      │
│        │                                         │      │
│        │  📧 Email:     [john@company.com    ]  │      │
│        │                                         │      │
│        │  🔒 Password:  [••••••••••••••••••••]  │      │
│        │                                         │      │
│        │                                         │      │
│        │    ✅ [Create User]  🔙 [Back to Login] │      │
│        │                                         │      │
│        │   ℹ️ New users are created as Operators │      │
│        │                                         │      │
│        └─────────────────────────────────────────┘      │
│                                                         │
│           ✅ User 'john_doe' created successfully!      │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

## 📊 **User Management Table**

```
┌────────────────────────────────────────────────────────────────────────────┐
│                        👥 User Management                                  │
│                                                                            │
│ ┌────────────────────────────────────────────────────────────────────────┐ │
│ │Username│    Email        │    Role    │Status │Last Login  │Actions   │ │
│ ├────────┼─────────────────┼────────────┼───────┼────────────┼──────────┤ │
│ │admin   │admin@hdd.local  │Administrator│Active │2025-09-26  │    -     │ │
│ │john_doe│john@company.com │Operator    │Active │Never       │Disable Delete│
│ │jane_v  │jane@company.com │Viewer      │Disabled│2025-09-25  │Enable  Delete│
│ │tech_01 │tech@company.com │Operator    │Active │2025-09-26  │Disable Delete│
│ └────────┴─────────────────┴────────────┴───────┴────────────┴──────────┘ │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

## 🖥️ **Main Application with Authentication**

```
┌────────────────────────────────────────────────────────────────────────────┐
│ SHREDX              👤 admin (Administrator)  👥 Users  🔄  🚪 Logout     │
│────────────────────────────────────────────────────────────────────────────│
│                                                                            │
│  [Drives] [Details] [Report]                                               │
│                                                                            │
│  DRIVES                                                                    │
│  ┌──────────────────────────────────────────────────────────────────────┐ │
│  │Select │Drive name│Drive path│Size │Used │Progress│Time left│Speed   │ │
│  ├───────┼──────────┼──────────┼─────┼─────┼────────┼─────────┼────────┤ │
│  │  ☑    │System-C  │   C:\    │500GB│300GB│  ████  │Complete │120MB/s │ │
│  │  ☐    │Data-D    │   D:\    │1TB  │750GB│   --   │   --    │  --    │ │
│  └───────┴──────────┴──────────┴─────┴─────┴────────┴─────────┴────────┘ │
│                                                                            │
│  ✓ Select All                                                              │
│                                                                            │
│  ADVANCE OPTIONS                                                           │
│  Eraser method: [NIST SP 800-88 and DoD 5220.22-M ▼     ]                │
│  Verification:  [json ▼]                                                  │
│                                                                            │
│  ✅ Confirm to erase the data                                              │
│  [    ERASE    ]                                                           │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

## 🚫 **Permission Denied State (Viewer Role)**

```
┌────────────────────────────────────────────────────────────────────────────┐
│ SHREDX                 👤 jane_viewer (Viewer)           🔄  🚪 Logout     │
│────────────────────────────────────────────────────────────────────────────│
│                                                                            │
│  [Drives] [Details] [Report]                                               │
│                                                                            │
│  ADVANCE OPTIONS                                                           │
│  Eraser method: [NIST SP 800-88 and DoD 5220.22-M ▼     ]                │
│  Verification:  [json ▼]                                                  │
│                                                                            │
│  🚫 Viewer role cannot perform sanitization                               │
│  ☐ Confirm to erase the data                                              │
│  [    ERASE    ] (disabled)                                               │
│  ⚠ Contact Administrator for sanitization permissions                      │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

## 🎨 **Color Scheme & Theme**

### **Authentication Pages**
- **Background**: Dark blue gradient (#0F172A to #1E293B)
- **Forms**: Semi-transparent dark panels (#1E293B80)
- **Buttons**: 
  - Login: Blue (#2563EB)
  - Create: Green (#059669)  
  - Cancel: Gray (#6B7280)
- **Text**: White (#FFFFFF) with gray hints (#9CA3AF)
- **Icons**: Colored emojis for visual appeal

### **Main Application**
- **Header**: Dark theme with user info
- **Success Messages**: Green (#22C55E)
- **Error Messages**: Red (#EF4444)
- **Warning Messages**: Yellow (#EAB308)
- **Permission Denied**: Red background with warning icon

## 📱 **Responsive Design Features**

- **Flexible layouts** that work on different screen sizes
- **Consistent spacing** with proper margins and padding  
- **Clear visual hierarchy** with proper font sizes
- **Accessible colors** with good contrast ratios
- **Intuitive navigation** with breadcrumbs and back buttons

## 🔧 **Interactive Elements**

- **Hover effects** on buttons and clickable elements
- **Focus states** for keyboard navigation
- **Loading indicators** during authentication
- **Real-time validation** with immediate feedback
- **Smooth transitions** between authentication states

## 🛡️ **Security Visual Indicators**

- **Lock icons** for password fields
- **Shield branding** for security emphasis
- **Role badges** showing user permissions
- **Status indicators** (Active/Disabled) with color coding
- **Permission warnings** with clear messaging

This authentication system provides a professional, secure, and user-friendly experience that matches enterprise security standards while maintaining the clean, modern aesthetic of the SHREDX brand! 🎉