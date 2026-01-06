import {
    CLPublicKey,
    CLValueBuilder,
    RuntimeArgs,
    CLAccountHash,
    DeployUtil,
} from 'casper-js-sdk';
import { config } from '../config';
import { deployService } from './deploy.service';
import { DeployResult } from '../types';

export class ContractService {
    private contractHash: string;

    constructor() {
        if (!config.contract.recoveryRegistryHash) {
            throw new Error('RECOVERY_REGISTRY_HASH is not defined in environment variables');
        }
        this.contractHash = config.contract.recoveryRegistryHash;
    }

    /**
     * Helper: Build a contract call deploy
     */
    private buildContractDeploy(
        callerPublicKey: CLPublicKey,
        entryPoint: string,
        args: RuntimeArgs,
        paymentAmount: string = config.deploy.paymentAmount
    ): DeployUtil.Deploy {
        return deployService.buildContractCallDeploy(
            callerPublicKey,
            this.contractHash,
            entryPoint,
            args,
            paymentAmount
        );
    }

    // ============================================================================
    // Initialize Guardians
    // ============================================================================

    /**
     * Register account with guardians and threshold
     * Entry Point: init_guardians
     * 
     * Uses stored contract call so data is stored in contract's dictionary
     * (same location where start_recovery looks for it)
     */
    async initializeGuardians(
        userPublicKeyHex: string,
        guardians: string[],
        threshold: number
    ): Promise<DeployResult> {
        const userPublicKey = CLPublicKey.fromHex(userPublicKeyHex);

        // Contract expects account as ByteArray(32), not Key type
        const userAccountHash = new CLAccountHash(userPublicKey.toAccountHash());

        const guardianAccountHashes = guardians.map((g) => {
            const pk = CLPublicKey.fromHex(g);
            return new CLAccountHash(pk.toAccountHash());
        });

        const args = RuntimeArgs.fromMap({
            account: userAccountHash,
            guardians: CLValueBuilder.list(guardianAccountHashes),
            threshold: CLValueBuilder.u8(threshold),
        });

        // Use stored contract call so data is stored in contract's dictionary
        // This is where start_recovery will look for the guardian data
        const deploy = this.buildContractDeploy(
            userPublicKey,
            'init_guardians',
            args
        );

        return {
            deployHash: '',
            success: true,
            message: deployService.deployToJson(deploy),
        };
    }

    // ============================================================================
    // Initiate Recovery
    // ============================================================================

    /**
     * Entry Point: start_recovery
     */
    async initiateRecovery(
        initiatorPublicKeyHex: string,
        targetAccountHex: string,
        newPublicKeyHex: string
    ): Promise<DeployResult> {
        const initiatorKey = CLPublicKey.fromHex(initiatorPublicKeyHex);
        const targetAccount = CLPublicKey.fromHex(targetAccountHex);
        const newPublicKey = CLPublicKey.fromHex(newPublicKeyHex);

        // Contract expects account as ByteArray(32), not Key type
        const targetAccountHash = new CLAccountHash(targetAccount.toAccountHash());

        const args = RuntimeArgs.fromMap({
            account: targetAccountHash,
            new_key: newPublicKey,
        });

        const deploy = this.buildContractDeploy(
            initiatorKey,
            'start_recovery',
            args
        );

        return {
            deployHash: '',
            success: true,
            message: deployService.deployToJson(deploy),
        };
    }

    // ============================================================================
    // Approve Recovery
    // ============================================================================

    /**
     * Entry Point: approve
     */
    async approveRecovery(
        guardianPublicKeyHex: string,
        recoveryId: string
    ): Promise<DeployResult> {
        const guardianKey = CLPublicKey.fromHex(guardianPublicKeyHex);

        const args = RuntimeArgs.fromMap({
            id: CLValueBuilder.u256(recoveryId),
        });

        const deploy = this.buildContractDeploy(
            guardianKey,
            'approve',
            args
        );

        return {
            deployHash: '',
            success: true,
            message: deployService.deployToJson(deploy),
        };
    }

    // ============================================================================
    // Check Threshold Met
    // ============================================================================

    /**
     * Entry Point: is_approved
     */
    async buildCheckThresholdDeploy(
        signerPublicKeyHex: string,
        recoveryId: string
    ): Promise<DeployResult> {
        const signerKey = CLPublicKey.fromHex(signerPublicKeyHex);

        const args = RuntimeArgs.fromMap({
            id: CLValueBuilder.u256(recoveryId),
        });

        const deploy = this.buildContractDeploy(
            signerKey,
            'is_approved',
            args,
            config.deploy.paymentAmount
        );

        return {
            deployHash: '',
            success: true,
            message: deployService.deployToJson(deploy),
        };
    }

    // ============================================================================
    // Finalize Recovery
    // ============================================================================

    /**
     * Entry Point: finalize
     */
    async finalizeRecovery(
        signerPublicKeyHex: string,
        recoveryId: string
    ): Promise<DeployResult> {
        const signerKey = CLPublicKey.fromHex(signerPublicKeyHex);

        const args = RuntimeArgs.fromMap({
            id: CLValueBuilder.u256(recoveryId),
        });

        const deploy = this.buildContractDeploy(
            signerKey,
            'finalize',
            args
        );

        return {
            deployHash: '',
            success: true,
            message: deployService.deployToJson(deploy),
        };
    }

    // ============================================================================
    // ACTION 8: Has Guardians
    // ============================================================================

    /**
     * Entry Point: has_guardians
     */
    async buildHasGuardiansDeploy(
        signerPublicKeyHex: string,
        targetAccountHex: string
    ): Promise<DeployResult> {
        const signerKey = CLPublicKey.fromHex(signerPublicKeyHex);
        const targetAccount = CLPublicKey.fromHex(targetAccountHex);
        const targetAccountHash = new CLAccountHash(targetAccount.toAccountHash());

        const args = RuntimeArgs.fromMap({
            account: CLValueBuilder.key(targetAccountHash),
        });

        const deploy = this.buildContractDeploy(
            signerKey,
            'has_guardians',
            args,
            config.deploy.paymentAmount
        );

        return {
            deployHash: '',
            success: true,
            message: deployService.deployToJson(deploy),
        };
    }

    /**
     * Entry Point: get_guardians
     */
    async buildGetGuardiansDeploy(
        signerPublicKeyHex: string,
        targetAccountHex: string
    ): Promise<DeployResult> {
        const signerKey = CLPublicKey.fromHex(signerPublicKeyHex);
        const targetAccount = CLPublicKey.fromHex(targetAccountHex);
        const targetAccountHash = new CLAccountHash(targetAccount.toAccountHash());

        const args = RuntimeArgs.fromMap({
            account: CLValueBuilder.key(targetAccountHash),
        });

        const deploy = this.buildContractDeploy(
            signerKey,
            'get_guardians',
            args,
            config.deploy.paymentAmount
        );

        return {
            deployHash: '',
            success: true,
            message: deployService.deployToJson(deploy),
        };
    }
}

export const contractService = new ContractService();
