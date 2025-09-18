use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::{info, warn, error};
use intelligence_core::{
    IntelligenceError, Result as IntelligenceResult, IntelligenceId
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingAccount {
    pub account_id: IntelligenceId,
    pub customer_id: IntelligenceId,
    pub subscription_id: Option<IntelligenceId>,
    pub billing_cycle: BillingCycle,
    pub payment_method: PaymentMethod,
    pub billing_address: BillingAddress,
    pub tax_information: TaxInformation,
    pub credit_limit: f64,
    pub current_balance: f64,
    pub status: BillingStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BillingCycle {
    Monthly,
    Quarterly,
    Annually,
    PayAsYouGo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethod {
    pub method_type: PaymentMethodType,
    pub card_last_four: Option<String>,
    pub card_brand: Option<String>,
    pub expiry_month: Option<u8>,
    pub expiry_year: Option<u16>,
    pub bank_account_last_four: Option<String>,
    pub bank_name: Option<String>,
    pub is_default: bool,
    pub is_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentMethodType {
    CreditCard,
    DebitCard,
    BankAccount,
    PayPal,
    Crypto,
    WireTransfer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingAddress {
    pub street: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxInformation {
    pub tax_id: Option<String>,
    pub tax_exempt: bool,
    pub tax_rate: f64,
    pub vat_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BillingStatus {
    Active,
    Suspended,
    Cancelled,
    PastDue,
    Inactive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub subscription_id: IntelligenceId,
    pub customer_id: IntelligenceId,
    pub plan_id: IntelligenceId,
    pub plan_name: String,
    pub status: SubscriptionStatus,
    pub billing_cycle: BillingCycle,
    pub price_per_cycle: f64,
    pub usage_limits: UsageLimits,
    pub current_usage: CurrentUsage,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub next_billing_date: DateTime<Utc>,
    pub auto_renew: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubscriptionStatus {
    Active,
    Cancelled,
    Expired,
    Suspended,
    Pending,
    Trial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageLimits {
    pub api_calls_per_month: u64,
    pub data_processing_gb: f64,
    pub storage_gb: f64,
    pub concurrent_users: u32,
    pub ml_inferences_per_month: u64,
    pub support_level: SupportLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SupportLevel {
    Basic,
    Standard,
    Premium,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUsage {
    pub api_calls_this_month: u64,
    pub data_processed_gb: f64,
    pub storage_used_gb: f64,
    pub active_users: u32,
    pub ml_inferences_this_month: u64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub invoice_id: IntelligenceId,
    pub customer_id: IntelligenceId,
    pub subscription_id: Option<IntelligenceId>,
    pub invoice_number: String,
    pub status: InvoiceStatus,
    pub subtotal: f64,
    pub tax_amount: f64,
    pub total_amount: f64,
    pub currency: String,
    pub due_date: DateTime<Utc>,
    pub paid_date: Option<DateTime<Utc>>,
    pub line_items: Vec<InvoiceLineItem>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InvoiceStatus {
    Draft,
    Sent,
    Paid,
    Overdue,
    Cancelled,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLineItem {
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub total_price: f64,
    pub tax_rate: f64,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingPlan {
    pub plan_id: IntelligenceId,
    pub name: String,
    pub description: String,
    pub price_per_month: f64,
    pub billing_cycles: Vec<BillingCycle>,
    pub features: Vec<String>,
    pub usage_limits: UsageLimits,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingSystem {
    pub billing_accounts: Arc<RwLock<HashMap<IntelligenceId, BillingAccount>>>,
    pub subscriptions: Arc<RwLock<HashMap<IntelligenceId, Subscription>>>,
    pub invoices: Arc<RwLock<HashMap<IntelligenceId, Invoice>>>,
    pub pricing_plans: Arc<RwLock<HashMap<IntelligenceId, PricingPlan>>>,
    pub usage_tracking: Arc<RwLock<HashMap<IntelligenceId, UsageTracking>>>,
    pub payment_processor: PaymentProcessor,
    pub is_running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageTracking {
    pub customer_id: IntelligenceId,
    pub subscription_id: Option<IntelligenceId>,
    pub current_usage: CurrentUsage,
    pub usage_history: Vec<UsageSnapshot>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSnapshot {
    pub timestamp: DateTime<Utc>,
    pub usage: CurrentUsage,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentProcessor {
    pub processor_type: PaymentProcessorType,
    pub api_key: String,
    pub webhook_secret: String,
    pub is_test_mode: bool,
    pub supported_currencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentProcessorType {
    Stripe,
    PayPal,
    Square,
    Braintree,
    Custom,
}

impl BillingSystem {
    pub fn new(payment_processor: PaymentProcessor) -> Self {
        Self {
            billing_accounts: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            invoices: Arc::new(RwLock::new(HashMap::new())),
            pricing_plans: Arc::new(RwLock::new(HashMap::new())),
            usage_tracking: Arc::new(RwLock::new(HashMap::new())),
            payment_processor,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn create_billing_account(
        &self,
        customer_id: IntelligenceId,
        billing_cycle: BillingCycle,
        payment_method: PaymentMethod,
        billing_address: BillingAddress,
    ) -> IntelligenceResult<BillingAccount> {
        let account = BillingAccount {
            account_id: IntelligenceId::new(),
            customer_id,
            subscription_id: None,
            billing_cycle,
            payment_method,
            billing_address,
            tax_information: TaxInformation {
                tax_id: None,
                tax_exempt: false,
                tax_rate: 0.0,
                vat_number: None,
            },
            credit_limit: 10000.0,
            current_balance: 0.0,
            status: BillingStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut accounts = self.billing_accounts.write().await;
        accounts.insert(account.account_id.clone(), account.clone());
        info!("Created billing account: {}", account.account_id);
        Ok(account)
    }

    pub async fn create_subscription(
        &self,
        customer_id: IntelligenceId,
        plan_id: IntelligenceId,
        billing_cycle: BillingCycle,
    ) -> IntelligenceResult<Subscription> {
        let plans = self.pricing_plans.read().await;
        let plan = plans.get(&plan_id)
            .ok_or_else(|| IntelligenceError::NotFound { 
                resource: "pricing_plan".to_string(),
                id: plan_id.clone(),
            })?;

        let now = Utc::now();
        let next_billing = match billing_cycle {
            BillingCycle::Monthly => now + chrono::Duration::days(30),
            BillingCycle::Quarterly => now + chrono::Duration::days(90),
            BillingCycle::Annually => now + chrono::Duration::days(365),
            BillingCycle::PayAsYouGo => now + chrono::Duration::days(1),
        };

        let subscription = Subscription {
            subscription_id: IntelligenceId::new(),
            customer_id,
            plan_id,
            plan_name: plan.name.clone(),
            status: SubscriptionStatus::Active,
            billing_cycle,
            price_per_cycle: plan.price_per_month,
            usage_limits: plan.usage_limits.clone(),
            current_usage: CurrentUsage {
                api_calls_this_month: 0,
                data_processed_gb: 0.0,
                storage_used_gb: 0.0,
                active_users: 0,
                ml_inferences_this_month: 0,
                last_updated: now,
            },
            start_date: now,
            end_date: None,
            next_billing_date: next_billing,
            auto_renew: true,
            created_at: now,
            updated_at: now,
        };

        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(subscription.subscription_id.clone(), subscription.clone());

        let mut accounts = self.billing_accounts.write().await;
        if let Some(account) = accounts.get_mut(&customer_id) {
            account.subscription_id = Some(subscription.subscription_id.clone());
        }

        info!("Created subscription: {}", subscription.subscription_id);
        Ok(subscription)
    }

    pub async fn track_usage(
        &self,
        customer_id: IntelligenceId,
        usage_type: UsageType,
        amount: f64,
    ) -> IntelligenceResult<()> {
        let mut tracking = self.usage_tracking.write().await;
        let usage_entry = tracking.entry(customer_id.clone()).or_insert_with(|| {
            UsageTracking {
                customer_id: customer_id.clone(),
                subscription_id: None,
                current_usage: CurrentUsage {
                    api_calls_this_month: 0,
                    data_processed_gb: 0.0,
                    storage_used_gb: 0.0,
                    active_users: 0,
                    ml_inferences_this_month: 0,
                    last_updated: Utc::now(),
                },
                usage_history: Vec::new(),
                last_updated: Utc::now(),
            }
        });

        match usage_type {
            UsageType::ApiCall => {
                usage_entry.current_usage.api_calls_this_month += amount as u64;
            }
            UsageType::DataProcessing => {
                usage_entry.current_usage.data_processed_gb += amount;
            }
            UsageType::Storage => {
                usage_entry.current_usage.storage_used_gb += amount;
            }
            UsageType::MLInference => {
                usage_entry.current_usage.ml_inferences_this_month += amount as u64;
            }
        }

        usage_entry.last_updated = Utc::now();
        info!("Tracked usage for customer {}: {:?} = {}", customer_id, usage_type, amount);
        Ok(())
    }

    pub async fn generate_invoice(
        &self,
        customer_id: IntelligenceId,
        billing_period_start: DateTime<Utc>,
        billing_period_end: DateTime<Utc>,
    ) -> IntelligenceResult<Invoice> {
        let subscriptions = self.subscriptions.read().await;
        let usage_tracking = self.usage_tracking.read().await;

        let customer_subscriptions: Vec<&Subscription> = subscriptions.values()
            .filter(|sub| sub.customer_id == customer_id && sub.status == SubscriptionStatus::Active)
            .collect();

        if customer_subscriptions.is_empty() {
            return Err(IntelligenceError::NotFound { 
                resource: "active_subscription".to_string(),
                id: customer_id,
            });
        }

        let mut line_items = Vec::new();
        let mut subtotal = 0.0;

        for subscription in customer_subscriptions {
            let base_price = subscription.price_per_cycle;
            line_items.push(InvoiceLineItem {
                description: format!("{} - {}", subscription.plan_name, subscription.billing_cycle),
                quantity: 1.0,
                unit_price: base_price,
                total_price: base_price,
                tax_rate: 0.0,
                category: "subscription".to_string(),
            });
            subtotal += base_price;

            if let Some(usage) = usage_tracking.get(&customer_id) {
                let overage_cost = self.calculate_overage_cost(subscription, &usage.current_usage).await?;
                if overage_cost > 0.0 {
                    line_items.push(InvoiceLineItem {
                        description: "Overage charges".to_string(),
                        quantity: 1.0,
                        unit_price: overage_cost,
                        total_price: overage_cost,
                        tax_rate: 0.0,
                        category: "overage".to_string(),
                    });
                    subtotal += overage_cost;
                }
            }
        }

        let tax_amount = subtotal * 0.08;
        let total_amount = subtotal + tax_amount;

        let invoice = Invoice {
            invoice_id: IntelligenceId::new(),
            customer_id,
            subscription_id: customer_subscriptions[0].subscription_id.clone(),
            invoice_number: format!("INV-{}", Utc::now().format("%Y%m%d-%H%M%S")),
            status: InvoiceStatus::Draft,
            subtotal,
            tax_amount,
            total_amount,
            currency: "USD".to_string(),
            due_date: Utc::now() + chrono::Duration::days(30),
            paid_date: None,
            line_items,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut invoices = self.invoices.write().await;
        invoices.insert(invoice.invoice_id.clone(), invoice.clone());
        info!("Generated invoice: {}", invoice.invoice_id);
        Ok(invoice)
    }

    pub async fn process_payment(
        &self,
        invoice_id: IntelligenceId,
        payment_method: PaymentMethod,
    ) -> IntelligenceResult<PaymentResult> {
        let mut invoices = self.invoices.write().await;
        let invoice = invoices.get_mut(&invoice_id)
            .ok_or_else(|| IntelligenceError::NotFound { 
                resource: "invoice".to_string(),
                id: invoice_id.clone(),
            })?;

        if invoice.status != InvoiceStatus::Sent && invoice.status != InvoiceStatus::Overdue {
            return Err(IntelligenceError::InvalidOperation { 
                message: "Invoice is not in a payable state".to_string(),
            });
        }

        let payment_result = self.process_payment_with_processor(
            &payment_method,
            invoice.total_amount,
            &invoice.currency,
        ).await?;

        if payment_result.success {
            invoice.status = InvoiceStatus::Paid;
            invoice.paid_date = Some(Utc::now());
            invoice.updated_at = Utc::now();

            let mut accounts = self.billing_accounts.write().await;
            if let Some(account) = accounts.get_mut(&invoice.customer_id) {
                account.current_balance -= invoice.total_amount;
                account.updated_at = Utc::now();
            }

            info!("Payment processed successfully for invoice: {}", invoice_id);
        } else {
            warn!("Payment failed for invoice {}: {}", invoice_id, payment_result.error_message);
        }

        Ok(payment_result)
    }

    pub async fn get_billing_summary(&self, customer_id: IntelligenceId) -> IntelligenceResult<BillingSummary> {
        let accounts = self.billing_accounts.read().await;
        let subscriptions = self.subscriptions.read().await;
        let invoices = self.invoices.read().await;
        let usage_tracking = self.usage_tracking.read().await;

        let account = accounts.get(&customer_id)
            .ok_or_else(|| IntelligenceError::NotFound { 
                resource: "billing_account".to_string(),
                id: customer_id.clone(),
            })?;

        let customer_subscriptions: Vec<&Subscription> = subscriptions.values()
            .filter(|sub| sub.customer_id == customer_id)
            .collect();

        let customer_invoices: Vec<&Invoice> = invoices.values()
            .filter(|inv| inv.customer_id == customer_id)
            .collect();

        let total_invoiced = customer_invoices.iter()
            .map(|inv| inv.total_amount)
            .sum();

        let total_paid = customer_invoices.iter()
            .filter(|inv| inv.status == InvoiceStatus::Paid)
            .map(|inv| inv.total_amount)
            .sum();

        let outstanding_balance = total_invoiced - total_paid;

        let current_usage = usage_tracking.get(&customer_id)
            .map(|tracking| tracking.current_usage.clone())
            .unwrap_or_else(|| CurrentUsage {
                api_calls_this_month: 0,
                data_processed_gb: 0.0,
                storage_used_gb: 0.0,
                active_users: 0,
                ml_inferences_this_month: 0,
                last_updated: Utc::now(),
            });

        Ok(BillingSummary {
            customer_id,
            account_status: account.status.clone(),
            current_balance: account.current_balance,
            credit_limit: account.credit_limit,
            active_subscriptions: customer_subscriptions.len(),
            total_invoiced,
            total_paid,
            outstanding_balance,
            current_usage,
            next_billing_date: customer_subscriptions.iter()
                .map(|sub| sub.next_billing_date)
                .min()
                .unwrap_or(Utc::now()),
            last_updated: Utc::now(),
        })
    }

    async fn calculate_overage_cost(
        &self,
        subscription: &Subscription,
        usage: &CurrentUsage,
    ) -> IntelligenceResult<f64> {
        let mut overage_cost = 0.0;

        if usage.api_calls_this_month > subscription.usage_limits.api_calls_per_month {
            let overage_calls = usage.api_calls_this_month - subscription.usage_limits.api_calls_per_month;
            overage_cost += overage_calls as f64 * 0.001;
        }

        if usage.data_processed_gb > subscription.usage_limits.data_processing_gb {
            let overage_gb = usage.data_processed_gb - subscription.usage_limits.data_processing_gb;
            overage_cost += overage_gb * 0.10;
        }

        if usage.storage_used_gb > subscription.usage_limits.storage_gb {
            let overage_gb = usage.storage_used_gb - subscription.usage_limits.storage_gb;
            overage_cost += overage_gb * 0.05;
        }

        if usage.ml_inferences_this_month > subscription.usage_limits.ml_inferences_per_month {
            let overage_inferences = usage.ml_inferences_this_month - subscription.usage_limits.ml_inferences_per_month;
            overage_cost += overage_inferences as f64 * 0.01;
        }

        Ok(overage_cost)
    }

    async fn process_payment_with_processor(
        &self,
        payment_method: &PaymentMethod,
        amount: f64,
        currency: &str,
    ) -> IntelligenceResult<PaymentResult> {
        match self.payment_processor.processor_type {
            PaymentProcessorType::Stripe => {
                self.process_stripe_payment(payment_method, amount, currency).await
            }
            PaymentProcessorType::PayPal => {
                self.process_paypal_payment(payment_method, amount, currency).await
            }
            _ => {
                Ok(PaymentResult {
                    success: true,
                    transaction_id: format!("TXN-{}", Utc::now().timestamp()),
                    error_message: None,
                    processed_at: Utc::now(),
                })
            }
        }
    }

    async fn process_stripe_payment(
        &self,
        _payment_method: &PaymentMethod,
        _amount: f64,
        _currency: &str,
    ) -> IntelligenceResult<PaymentResult> {
        Ok(PaymentResult {
            success: true,
            transaction_id: format!("stripe-{}", Utc::now().timestamp()),
            error_message: None,
            processed_at: Utc::now(),
        })
    }

    async fn process_paypal_payment(
        &self,
        _payment_method: &PaymentMethod,
        _amount: f64,
        _currency: &str,
    ) -> IntelligenceResult<PaymentResult> {
        Ok(PaymentResult {
            success: true,
            transaction_id: format!("paypal-{}", Utc::now().timestamp()),
            error_message: None,
            processed_at: Utc::now(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UsageType {
    ApiCall,
    DataProcessing,
    Storage,
    MLInference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResult {
    pub success: bool,
    pub transaction_id: String,
    pub error_message: Option<String>,
    pub processed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingSummary {
    pub customer_id: IntelligenceId,
    pub account_status: BillingStatus,
    pub current_balance: f64,
    pub credit_limit: f64,
    pub active_subscriptions: usize,
    pub total_invoiced: f64,
    pub total_paid: f64,
    pub outstanding_balance: f64,
    pub current_usage: CurrentUsage,
    pub next_billing_date: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}
