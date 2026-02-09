package com.digitalpebble.ginkgo.model;

import com.google.gson.annotations.SerializedName;

public class UsageItem {

    private String date;
    private String product;
    private String sku;
    private double quantity;
    private String unitType;
    private double pricePerUnit;
    private double grossAmount;
    private double discountAmount;
    private double netAmount;
    private String organizationName;
    private String repositoryName;

    @SerializedName("energy_usage_wh")
    private Double energyUsageWh;

    @SerializedName("co2eq_g")
    private Double co2eqG;

    public String getDate() {
        return date;
    }

    public void setDate(String date) {
        this.date = date;
    }

    public String getProduct() {
        return product;
    }

    public void setProduct(String product) {
        this.product = product;
    }

    public String getSku() {
        return sku;
    }

    public void setSku(String sku) {
        this.sku = sku;
    }

    public double getQuantity() {
        return quantity;
    }

    public void setQuantity(double quantity) {
        this.quantity = quantity;
    }

    public String getUnitType() {
        return unitType;
    }

    public void setUnitType(String unitType) {
        this.unitType = unitType;
    }

    public double getPricePerUnit() {
        return pricePerUnit;
    }

    public void setPricePerUnit(double pricePerUnit) {
        this.pricePerUnit = pricePerUnit;
    }

    public double getGrossAmount() {
        return grossAmount;
    }

    public void setGrossAmount(double grossAmount) {
        this.grossAmount = grossAmount;
    }

    public double getDiscountAmount() {
        return discountAmount;
    }

    public void setDiscountAmount(double discountAmount) {
        this.discountAmount = discountAmount;
    }

    public double getNetAmount() {
        return netAmount;
    }

    public void setNetAmount(double netAmount) {
        this.netAmount = netAmount;
    }

    public String getOrganizationName() {
        return organizationName;
    }

    public void setOrganizationName(String organizationName) {
        this.organizationName = organizationName;
    }

    public String getRepositoryName() {
        return repositoryName;
    }

    public void setRepositoryName(String repositoryName) {
        this.repositoryName = repositoryName;
    }

    public Double getEnergyUsageWh() { return energyUsageWh; }
    public void setEnergyUsageWh(Double energyUsageWh) { this.energyUsageWh = energyUsageWh; }

    public Double getCo2eqG() { return co2eqG; }
    public void setCo2eqG(Double co2eqG) { this.co2eqG = co2eqG; }
}
