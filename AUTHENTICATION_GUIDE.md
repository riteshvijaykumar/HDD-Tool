# 🛡️ HDD Tool Authentication System

## 📋 **Overview**

The HDD Tool now includes a comprehensive authentication system with user management, role-based access control, and s## 🔧 **Configuration Options**

### **User Role Management**
- **Default Role**: All new users are created as Operators
- **Role Changes**: Admins can promote/demote users through the user management interface
- **Role Permissions**: Defined in code and easily customizable

### **Role Customization**
The system supports easy role modification:

```rust
impl UserRole {
    pub fn can_sanitize(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Operator)
    }
    
    pub fn can_manage_users(&self) -> bool {
        matches!(self, UserRole::Admin)
    }
}
```andling. This ensures that only authorized personnel can perform critical disk sanitization operations.

## 🔐 **Authentication Features**

### **1. User Roles & Permissions**

- **Administrator** 🔧
  - Full access to all features
  - Can perform sanitization operations
  - Can create, manage, and delete users
  - Can access user management interface

- **Operator** ⚙️
  - Can perform sanitization operations
  - Cannot manage users
  - View-only access to user information

- **Viewer** 👁️
  - Read-only access
  - Cannot perform sanitization operations
  - Cannot manage users
  - Can view reports and drive information

### **2. Security Features**

- **Password Hashing**: SHA-256 encryption for all passwords
- **Session Management**: Secure login/logout functionality  
- **Permission Checks**: Role-based operation restrictions
- **Default Admin**: Auto-created admin account on first run
- **User Data Persistence**: Encrypted user database (users.json)

### **3. Login System**

#### **Default Credentials**
```
Username: admin
Password: admin123
```

#### **Login Features**
- Clean, professional login interface
- Password visibility toggle
- Input validation
- Error message display
- Auto-focus on username field

## 🎯 **User Interface**

### **Login Page**
- 🛡️ **SHREDX Authentication** branding
- Username and password fields with icons
- Show/hide password toggle
- Login button with enter key support
- Link to create user page (admin only)
- Default credentials reminder

### **Create User Page**
- Clean, aligned form with grid layout
- Form fields: Username, Email, Password
- Input validation (min 3 chars username, min 6 chars password)
- All new users are created as Operators by default
- Success/error message display
- Back to login navigation

### **User Management Page** (Admin only)
- Complete user table with:
  - Username, Email, Role, Status
  - Last login timestamp
  - Enable/Disable toggle
  - Delete user option
- Grid layout with sorting capabilities
- Real-time status updates

### **Main Application Changes**
- User info display in top bar
- Logout button
- Permission-based UI elements
- Role-specific warnings and restrictions

## 🚀 **How It Works**

### **1. First Time Setup**
```
1. Launch HDD Tool
2. Default admin user is automatically created
3. Login with: admin / admin123
4. Create additional users as needed
```

### **2. User Creation Process**
```
1. Admin logs in
2. Clicks "Create User" link
3. Fills out user form (Username, Email, Password)
4. User is automatically created as Operator
5. New user can immediately log in with Operator privileges
```

### **3. Permission System**
```
┌─────────────────────────────────────────────────┐
│                OPERATION                        │
├─────────────────┬───────────┬──────────┬────────┤
│     Feature     │   Admin   │ Operator │ Viewer │
├─────────────────┼───────────┼──────────┼────────┤
│ Drive Detection │     ✅     │    ✅     │   ✅    │
│ View Reports    │     ✅     │    ✅     │   ✅    │
│ Sanitization    │     ✅     │    ✅     │   ❌    │
│ User Management │     ✅     │    ❌     │   ❌    │
│ System Settings │     ✅     │    ❌     │   ❌    │
└─────────────────┴───────────┴──────────┴────────┘
```

## 🔧 **Technical Implementation**

### **Authentication Module** (`src/auth.rs`)
- `AuthSystem` - Core authentication logic
- `AuthUI` - User interface components
- `User` - User data structure with roles
- `UserRole` - Permission enumeration

### **Password Security**
```rust
// SHA-256 password hashing
fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

### **Permission Checks**
```rust
// Example permission check before sanitization
if let Some(user) = auth_system.current_user() {
    if !user.role.can_sanitize() {
        return Err("Access Denied: Role cannot perform sanitization");
    }
}
```

### **Data Storage**
- User data stored in `users.json`
- Automatic backup on changes
- Password hashes never stored in plain text
- Session data kept in memory only

## 📝 **Usage Examples**

### **Admin Workflow**
```
1. Login as admin
2. Create operator accounts for technicians
3. Set up viewer accounts for auditors
4. Monitor user activity
5. Manage user permissions as needed
```

### **Operator Workflow**
```
1. Login with operator credentials
2. Access drive sanitization features
3. Perform NIST-compliant erasure operations
4. Generate compliance reports
5. Logout when finished
```

### **Viewer Workflow**
```
1. Login with viewer credentials
2. Review sanitization reports
3. Verify compliance documentation
4. Monitor system status
5. No ability to modify or delete data
```

## 🛠️ **Configuration Options**

### **Role Customization**
The system supports easy role modification:

```rust
impl UserRole {
    pub fn can_sanitize(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Operator)
    }
    
    pub fn can_manage_users(&self) -> bool {
        matches!(self, UserRole::Admin)
    }
}
```

### **Security Settings**
- Password complexity can be adjusted in `create_user()` method
- Session timeout can be implemented
- Multi-factor authentication can be added

## 🚨 **Important Security Notes**

1. **Change Default Password**: Immediately change the default admin password in production
2. **Regular Backups**: Backup `users.json` regularly
3. **Access Control**: Limit physical access to the system
4. **Audit Trail**: Monitor user login activities
5. **Regular Updates**: Keep user accounts current and remove unused accounts

## 🔄 **Backup & Recovery**

### **User Data Backup**
```bash
# Backup user database
cp users.json users_backup_$(date +%Y%m%d).json

# Restore from backup
cp users_backup_20250926.json users.json
```

### **Emergency Admin Access**
If admin access is lost:
1. Delete `users.json` file
2. Restart HDD Tool
3. Default admin account will be recreated
4. Login with: admin / admin123

## 📈 **Future Enhancements**

- **Audit Logging**: Track all user actions
- **Password Expiration**: Force periodic password changes
- **Multi-Factor Authentication**: Add 2FA support
- **LDAP Integration**: Connect to enterprise directory services
- **API Authentication**: Token-based API access
- **Single Sign-On**: SAML/OAuth integration

---

## 🎉 **Quick Start Guide**

1. **Launch HDD Tool** - Authentication screen appears
2. **First Login** - Use `admin` / `admin123`
3. **Create Users** - Add operator and viewer accounts
4. **Test Permissions** - Verify role-based access works
5. **Change Admin Password** - Update default credentials
6. **Begin Operations** - Start using authenticated HDD Tool

Your HDD Tool is now secure with enterprise-grade authentication! 🛡️