# AWS Lambda Deployment Guide (Free Tier)

Complete step-by-step guide to deploy this app on AWS Lambda — **zero AWS knowledge required**.

## What You'll Get (Free Tier)

| Service | Free Allowance | What It Does |
|---------|---------------|--------------|
| **Lambda** | 1M requests/month (forever) | Runs your Rust code |
| **API Gateway** | 1M requests/month (12 months) | Gives you an HTTPS URL |
| **RDS PostgreSQL** | 750 hours/month (12 months) | Your database |
| **CloudWatch Logs** | 5 GB/month (forever) | View your app logs |

**Estimated cost: $0/month** (within free tier limits)

---

## Step 1: Create an IAM User for Deployment

You need an AWS user with permissions to deploy. **Don't use your root account.**

1. Go to **AWS Console** → search **"IAM"** → click **IAM**
2. Click **"Users"** in the left sidebar
3. Click **"Create user"**
4. **User name:** `github-deployer`
5. Click **"Next"**
6. Select **"Attach policies directly"**
7. Search and check these policies:
   - `AWSLambda_FullAccess`
   - `AmazonAPIGatewayAdministrator`
   - `AWSCloudFormationFullAccess`
   - `IAMFullAccess`
   - `AmazonS3FullAccess`
   - `CloudWatchLogsFullAccess`
8. Click **"Next"** → **"Create user"**
9. Click on the user you just created → **"Security credentials"** tab
10. Scroll down to **"Access keys"** → **"Create access key"**
11. Select **"Command Line Interface (CLI)"**
12. Check the confirmation box → **"Next"** → **"Create access key"**
13. **⚠️ SAVE BOTH KEYS NOW** — you won't see the secret key again:
    - `Access key ID` → save it (looks like `AKIAIOSFODNN7EXAMPLE`)
    - `Secret access key` → save it (looks like `wJalrXUtnFEMI/K7MDENG/...`)

---

## Step 2: Create the PostgreSQL Database (RDS)

1. Go to **AWS Console** → search **"RDS"** → click **RDS**
2. Click **"Create database"**
3. Fill in these settings:

| Setting | Value |
|---------|-------|
| **Creation method** | Standard create |
| **Engine type** | PostgreSQL |
| **Engine version** | PostgreSQL 16 (latest) |
| **Templates** | ⭐ **Free tier** |
| **DB instance identifier** | `notes-db` |
| **Master username** | `notesuser` |
| **Credentials management** | Self managed |
| **Master password** | Choose a strong password (save it!) |
| **DB instance class** | db.t3.micro (or db.t4g.micro) — should auto-select with Free tier |
| **Storage type** | gp2 |
| **Allocated storage** | 20 GB |
| **Storage autoscaling** | ❌ Uncheck "Enable storage autoscaling" |

4. Under **Connectivity**:

| Setting | Value |
|---------|-------|
| **Compute resource** | Don't connect to an EC2 compute resource |
| **Network type** | IPv4 |
| **VPC** | Default VPC |
| **Public access** | ⭐ **Yes** |
| **VPC security group** | Create new |
| **New VPC security group name** | `notes-db-sg` |

5. Under **Additional configuration**:

| Setting | Value |
|---------|-------|
| **Initial database name** | `notesdb` |
| **Automated backups** | ❌ Uncheck (saves storage costs) |
| **Encryption** | Leave default |
| **Monitoring** | ❌ Uncheck "Enable Enhanced monitoring" |

6. Click **"Create database"**
7. **Wait 5–10 minutes** for the database to be created (status: "Available")

---

## Step 3: Allow Connections to Your Database

Your database needs to accept connections from Lambda (which runs from AWS IP addresses).

1. Go to **RDS** → click on your database **"notes-db"**
2. Under **"Connectivity & security"**, find the **"VPC security groups"** link → click on **"notes-db-sg"**
3. Click the **"Security group ID"** (starts with `sg-`)
4. Click **"Edit inbound rules"**
5. You'll see an existing rule. **Modify it** or add a new one:

| Type | Protocol | Port range | Source |
|------|----------|-----------|--------|
| PostgreSQL | TCP | 5432 | **Anywhere-IPv4** (0.0.0.0/0) |

6. Click **"Save rules"**

> **⚠️ Security Note:** This allows connections from any IP. For a personal/learning project this is fine because your database still requires a username + password + SSL. For production, you'd use VPC + RDS Proxy instead.

---

## Step 4: Get Your Database Connection String

1. Go to **RDS** → click on **"notes-db"**
2. Under **"Connectivity & security"**, copy the **"Endpoint"** (looks like: `notes-db.c1234567890.ap-south-1.rds.amazonaws.com`)
3. Your connection string is:

```
postgresql://notesuser:YOUR_PASSWORD@YOUR_ENDPOINT:5432/notesdb?sslmode=require
```

**Example:**
```
postgresql://notesuser:MySecurePass123@notes-db.c9876543210.ap-south-1.rds.amazonaws.com:5432/notesdb?sslmode=require
```

**Save this connection string — you'll need it in Step 6.**

---

## Step 5: Run Database Migrations

Before deploying, your database tables need to be created. You can do this from your local machine:

1. Open PowerShell
2. Set the DATABASE_URL temporarily:

```powershell
$env:DATABASE_URL = "postgresql://notesuser:YOUR_PASSWORD@YOUR_ENDPOINT:5432/notesdb?sslmode=require"
$env:JWT_SECRET = "temp_secret_key_for_migration_only_32chars"
```

3. Run the app briefly (it will run migrations automatically and then you can stop it):

```powershell
cd "d:\Rust\Notes App\Rust backend"
cargo run
```

4. You should see:
```
Running database migrations...
Database migrations executed successfully.
🚀 Server running on http://0.0.0.0:3000
```

5. Press **Ctrl+C** to stop — the migrations have been applied!

---

## Step 6: Add Secrets to Your GitHub Repository

1. Go to your GitHub repository page
2. Click **"Settings"** (tab at the top)
3. Click **"Secrets and variables"** → **"Actions"** (left sidebar)
4. Click **"New repository secret"** and add each one:

| Secret Name | Value |
|------------|-------|
| `AWS_ACCESS_KEY_ID` | The access key from Step 1 |
| `AWS_SECRET_ACCESS_KEY` | The secret key from Step 1 |
| `AWS_REGION` | `ap-south-1` |
| `DATABASE_URL` | The connection string from Step 4 |
| `JWT_SECRET` | A random string, at least 32 characters. Example: `MyNotesAppJWTSecret2024SecureRandomKey!!` |

**That's it — only 5 secrets needed for free tier!**

---

## Step 7: Push to GitHub — Auto Deploy! 🚀

Now just push your code and GitHub Actions handles everything:

```powershell
cd "d:\Rust\Notes App\Rust backend"
git add .
git commit -m "Add AWS Lambda deployment"
git push origin main
```

### Watch the deployment:
1. Go to your GitHub repo → **"Actions"** tab
2. You'll see the workflow running: **"Deploy to AWS Lambda"**
3. Click on it to watch the progress
4. When it finishes (✅ green), you'll see your **API URL** in the job summary

The workflow does this automatically:
```
✅ Lint & type-check your code
🔨 Build the Rust binary for Lambda (Linux)
📦 Package with AWS SAM
🚀 Deploy to AWS Lambda + API Gateway
🏥 Health check the deployed API
📋 Print your API URL
```

---

## Step 8: Test Your Deployed API

After deployment succeeds, get your API URL from the GitHub Actions output. It looks like:
```
https://abc123xyz.execute-api.ap-south-1.amazonaws.com/prod
```

### Test with PowerShell:

```powershell
# Set your API URL (replace with yours)
$API = "https://abc123xyz.execute-api.ap-south-1.amazonaws.com/prod"

# 1. Health check
Invoke-RestMethod -Uri "$API/health"

# 2. Sign up a user
$body = '{"username":"testuser","email":"test@example.com","password":"securepass123"}'
Invoke-RestMethod -Uri "$API/auth/signup" -Method POST -Body $body -ContentType "application/json"

# 3. Log in
$body = '{"username":"testuser","password":"securepass123"}'
$response = Invoke-RestMethod -Uri "$API/auth/login" -Method POST -Body $body -ContentType "application/json"
$token = $response.token
Write-Host "Token: $token"

# 4. Create a note
$body = '{"title":"My First Note","content":"Deployed on Lambda!"}'
$headers = @{ "Authorization" = "Bearer $token" }
Invoke-RestMethod -Uri "$API/notes" -Method POST -Body $body -ContentType "application/json" -Headers $headers

# 5. List notes
Invoke-RestMethod -Uri "$API/notes" -Method GET -Headers $headers
```

---

## Step 9: View Logs (When Things Go Wrong)

### From GitHub Actions:
- Go to **Actions** tab → click on the failed run → read the error logs

### From AWS Console:
1. Go to **AWS Console** → search **"CloudWatch"**
2. Click **"Log groups"** in the left sidebar
3. Click on `/aws/lambda/notes-backend-prod`
4. Click on the latest **Log stream** to see your app's logs

---

## Step 10: Update Your Frontend

Update your Angular frontend to use the new API URL:

```typescript
// In your Angular environment file
export const environment = {
  production: true,
  apiUrl: 'https://abc123xyz.execute-api.ap-south-1.amazonaws.com/prod'
};
```

---

## How to Deploy Updates

After the initial setup, deploying updates is just:

```powershell
git add .
git commit -m "Your changes"
git push origin main
```

GitHub Actions automatically rebuilds and deploys. That's it! 🎉

---

## Manual Deploy (Without GitHub Actions)

If you prefer to deploy from your local machine:

### Install tools (one-time):
```powershell
# Install AWS CLI
winget install Amazon.AWSCLI

# Install SAM CLI
winget install Amazon.SAM-CLI

# Install cargo-lambda
cargo install cargo-lambda

# Configure AWS credentials
aws configure
# Enter: Access Key ID, Secret Key, Region (ap-south-1), Output (json)
```

### Deploy:
```powershell
cd "d:\Rust\Notes App\Rust backend"
.\deploy-lambda.ps1 -Guided
```

---

## Troubleshooting

### "Access Denied" during deploy
→ Your IAM user is missing permissions. Go back to Step 1 and make sure all 6 policies are attached.

### "Unable to connect to database"
→ Check that your RDS security group allows inbound on port 5432 from 0.0.0.0/0 (Step 3).
→ Verify your DATABASE_URL is correct (Step 4).
→ Make sure "Public access" is set to "Yes" on your RDS instance (Step 2).

### "Function timeout" (Lambda runs > 30 seconds)
→ Your database might be slow to connect. Check RDS is in the same region (ap-south-1).
→ Try increasing timeout in template.yaml: `Timeout: 60`

### "Cold start is slow"
→ First request after inactivity takes ~200ms extra (Rust is very fast). This is normal for free tier.
→ To eliminate cold starts, set `ProvisionedConcurrency` to 1+ (but this costs ~$3.50/month per instance).

### GitHub Actions build fails
→ Check the Actions tab for the specific error.
→ Most common: missing GitHub secrets (Step 6).

---

## Cost Summary (Free Tier)

| What You're Using | Monthly Cost |
|-------------------|-------------|
| Lambda (1M requests free) | **$0** |
| API Gateway (1M requests free) | **$0** |
| RDS db.t3.micro (750 hours free) | **$0** |
| RDS Storage 20 GB (free) | **$0** |
| CloudWatch Logs (5 GB free) | **$0** |
| **Total** | **$0/month** |

> **Note:** Free tier for RDS and API Gateway expires after **12 months**. Lambda's free tier is permanent. After 12 months, RDS costs ~$15/month and API Gateway ~$1/million requests.

---

## Cleanup (Delete Everything)

If you want to remove everything from AWS:

```powershell
# Delete the Lambda stack
aws cloudformation delete-stack --stack-name notes-backend-prod --region ap-south-1

# Delete the RDS database
aws rds delete-db-instance --db-instance-identifier notes-db --skip-final-snapshot --region ap-south-1
```

Or do it from the AWS Console:
1. **Lambda** → Functions → delete `notes-backend-prod`
2. **CloudFormation** → Stacks → delete `notes-backend-prod`
3. **RDS** → Databases → delete `notes-db` (skip final snapshot)
