(function() {var implementors = {
"fragnova_runtime":[["impl Decode for <a class=\"struct\" href=\"fragnova_runtime/opaque/struct.SessionKeys.html\" title=\"struct fragnova_runtime::opaque::SessionKeys\">SessionKeys</a>"],["impl Decode for <a class=\"enum\" href=\"fragnova_runtime/enum.RuntimeEvent.html\" title=\"enum fragnova_runtime::RuntimeEvent\">RuntimeEvent</a>"],["impl Decode for <a class=\"enum\" href=\"fragnova_runtime/enum.OriginCaller.html\" title=\"enum fragnova_runtime::OriginCaller\">OriginCaller</a>"],["impl Decode for <a class=\"enum\" href=\"fragnova_runtime/enum.RuntimeCall.html\" title=\"enum fragnova_runtime::RuntimeCall\">RuntimeCall</a>"]],
"pallet_accounts":[["impl Decode for <a class=\"struct\" href=\"pallet_accounts/crypto/struct.Public.html\" title=\"struct pallet_accounts::crypto::Public\">Public</a>"],["impl Decode for <a class=\"struct\" href=\"pallet_accounts/crypto/struct.Signature.html\" title=\"struct pallet_accounts::crypto::Signature\">Signature</a>"],["impl Decode for <a class=\"enum\" href=\"pallet_accounts/enum.ExternalID.html\" title=\"enum pallet_accounts::ExternalID\">ExternalID</a>"],["impl&lt;TPublic&gt; Decode for <a class=\"struct\" href=\"pallet_accounts/struct.EthLockUpdate.html\" title=\"struct pallet_accounts::EthLockUpdate\">EthLockUpdate</a>&lt;TPublic&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TPublic: Decode,</span>"],["impl&lt;TBalance, TBlockNum&gt; Decode for <a class=\"struct\" href=\"pallet_accounts/struct.EthLock.html\" title=\"struct pallet_accounts::EthLock\">EthLock</a>&lt;TBalance, TBlockNum&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TBalance: Decode,<br>&nbsp;&nbsp;&nbsp;&nbsp;TBlockNum: Decode,</span>"],["impl&lt;TAccountID, TMoment&gt; Decode for <a class=\"struct\" href=\"pallet_accounts/struct.AccountInfo.html\" title=\"struct pallet_accounts::AccountInfo\">AccountInfo</a>&lt;TAccountID, TMoment&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TAccountID: Decode,<br>&nbsp;&nbsp;&nbsp;&nbsp;TMoment: Decode,</span>"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"pallet_accounts/pallet/trait.Config.html\" title=\"trait pallet_accounts::pallet::Config\">Config</a>&gt; Decode for <a class=\"enum\" href=\"pallet_accounts/pallet/enum.Event.html\" title=\"enum pallet_accounts::pallet::Event\">Event</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T::AccountId: Decode,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;T as Config&gt;::Balance: Decode,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;T as Config&gt;::Balance: Decode,</span>"],["impl&lt;T&gt; Decode for <a class=\"enum\" href=\"pallet_accounts/pallet/enum.Error.html\" title=\"enum pallet_accounts::pallet::Error\">Error</a>&lt;T&gt;"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"pallet_accounts/pallet/trait.Config.html\" title=\"trait pallet_accounts::pallet::Config\">Config</a>&gt; Decode for <a class=\"enum\" href=\"pallet_accounts/pallet/enum.Call.html\" title=\"enum pallet_accounts::pallet::Call\">Call</a>&lt;T&gt;"]],
"pallet_aliases":[["impl&lt;TAccountId&gt; Decode for <a class=\"enum\" href=\"pallet_aliases/enum.LinkTarget.html\" title=\"enum pallet_aliases::LinkTarget\">LinkTarget</a>&lt;TAccountId&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TAccountId: Decode,</span>"],["impl&lt;TAccountId, TBlockNum&gt; Decode for <a class=\"struct\" href=\"pallet_aliases/struct.LinkTargetVersioned.html\" title=\"struct pallet_aliases::LinkTargetVersioned\">LinkTargetVersioned</a>&lt;TAccountId, TBlockNum&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"pallet_aliases/enum.LinkTarget.html\" title=\"enum pallet_aliases::LinkTarget\">LinkTarget</a>&lt;TAccountId&gt;: Decode,<br>&nbsp;&nbsp;&nbsp;&nbsp;TBlockNum: Decode,</span>"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"pallet_aliases/pallet/trait.Config.html\" title=\"trait pallet_aliases::pallet::Config\">Config</a>&gt; Decode for <a class=\"enum\" href=\"pallet_aliases/pallet/enum.Event.html\" title=\"enum pallet_aliases::pallet::Event\">Event</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T::AccountId: Decode,</span>"],["impl&lt;T&gt; Decode for <a class=\"enum\" href=\"pallet_aliases/pallet/enum.Error.html\" title=\"enum pallet_aliases::pallet::Error\">Error</a>&lt;T&gt;"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"pallet_aliases/pallet/trait.Config.html\" title=\"trait pallet_aliases::pallet::Config\">Config</a>&gt; Decode for <a class=\"enum\" href=\"pallet_aliases/pallet/enum.Call.html\" title=\"enum pallet_aliases::pallet::Call\">Call</a>&lt;T&gt;"]],
"pallet_clusters":[["impl Decode for <a class=\"struct\" href=\"pallet_clusters/struct.CompactSetting.html\" title=\"struct pallet_clusters::CompactSetting\">CompactSetting</a>"],["impl Decode for <a class=\"struct\" href=\"pallet_clusters/struct.Role.html\" title=\"struct pallet_clusters::Role\">Role</a>"],["impl&lt;TAccountId&gt; Decode for <a class=\"struct\" href=\"pallet_clusters/struct.Cluster.html\" title=\"struct pallet_clusters::Cluster\">Cluster</a>&lt;TAccountId&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TAccountId: Decode,</span>"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"pallet_clusters/pallet/trait.Config.html\" title=\"trait pallet_clusters::pallet::Config\">Config</a>&gt; Decode for <a class=\"enum\" href=\"pallet_clusters/pallet/enum.Event.html\" title=\"enum pallet_clusters::pallet::Event\">Event</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T::AccountId: Decode,</span>"],["impl&lt;T&gt; Decode for <a class=\"enum\" href=\"pallet_clusters/pallet/enum.Error.html\" title=\"enum pallet_clusters::pallet::Error\">Error</a>&lt;T&gt;"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"pallet_clusters/pallet/trait.Config.html\" title=\"trait pallet_clusters::pallet::Config\">Config</a>&gt; Decode for <a class=\"enum\" href=\"pallet_clusters/pallet/enum.Call.html\" title=\"enum pallet_clusters::pallet::Call\">Call</a>&lt;T&gt;"]],
"pallet_detach":[["impl Decode for <a class=\"struct\" href=\"pallet_detach/crypto/struct.Public.html\" title=\"struct pallet_detach::crypto::Public\">Public</a>"],["impl Decode for <a class=\"struct\" href=\"pallet_detach/crypto/struct.Signature.html\" title=\"struct pallet_detach::crypto::Signature\">Signature</a>"],["impl Decode for <a class=\"enum\" href=\"pallet_detach/enum.DetachHash.html\" title=\"enum pallet_detach::DetachHash\">DetachHash</a>"],["impl Decode for <a class=\"enum\" href=\"pallet_detach/enum.DetachCollectionType.html\" title=\"enum pallet_detach::DetachCollectionType\">DetachCollectionType</a>"],["impl Decode for <a class=\"enum\" href=\"pallet_detach/enum.DetachCollection.html\" title=\"enum pallet_detach::DetachCollection\">DetachCollection</a>"],["impl Decode for <a class=\"enum\" href=\"pallet_detach/enum.SupportedChains.html\" title=\"enum pallet_detach::SupportedChains\">SupportedChains</a>"],["impl Decode for <a class=\"struct\" href=\"pallet_detach/struct.DetachRequest.html\" title=\"struct pallet_detach::DetachRequest\">DetachRequest</a>"],["impl&lt;TPublic&gt; Decode for <a class=\"struct\" href=\"pallet_detach/struct.DetachInternalData.html\" title=\"struct pallet_detach::DetachInternalData\">DetachInternalData</a>&lt;TPublic&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TPublic: Decode,</span>"],["impl Decode for <a class=\"struct\" href=\"pallet_detach/struct.ExportData.html\" title=\"struct pallet_detach::ExportData\">ExportData</a>"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"pallet_detach/pallet/trait.Config.html\" title=\"trait pallet_detach::pallet::Config\">Config</a>&gt; Decode for <a class=\"enum\" href=\"pallet_detach/pallet/enum.Event.html\" title=\"enum pallet_detach::pallet::Event\">Event</a>&lt;T&gt;"],["impl&lt;T&gt; Decode for <a class=\"enum\" href=\"pallet_detach/pallet/enum.Error.html\" title=\"enum pallet_detach::pallet::Error\">Error</a>&lt;T&gt;"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"pallet_detach/pallet/trait.Config.html\" title=\"trait pallet_detach::pallet::Config\">Config</a>&gt; Decode for <a class=\"enum\" href=\"pallet_detach/pallet/enum.Call.html\" title=\"enum pallet_detach::pallet::Call\">Call</a>&lt;T&gt;"]],
"pallet_fragments":[["impl&lt;TAccountId, TString&gt; Decode for <a class=\"struct\" href=\"pallet_fragments/struct.GetDefinitionsParams.html\" title=\"struct pallet_fragments::GetDefinitionsParams\">GetDefinitionsParams</a>&lt;TAccountId, TString&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.68.0/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;TString&gt;: Decode,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.68.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;TAccountId&gt;: Decode,</span>"],["impl&lt;TAccountId, TString&gt; Decode for <a class=\"struct\" href=\"pallet_fragments/struct.GetInstancesParams.html\" title=\"struct pallet_fragments::GetInstancesParams\">GetInstancesParams</a>&lt;TAccountId, TString&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TString: Decode,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.68.0/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;TString&gt;: Decode,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.68.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;TAccountId&gt;: Decode,</span>"],["impl&lt;TString&gt; Decode for <a class=\"struct\" href=\"pallet_fragments/struct.GetInstanceOwnerParams.html\" title=\"struct pallet_fragments::GetInstanceOwnerParams\">GetInstanceOwnerParams</a>&lt;TString&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TString: Decode,</span>"],["impl&lt;TBlockNum&gt; Decode for <a class=\"struct\" href=\"pallet_fragments/struct.PublishingData.html\" title=\"struct pallet_fragments::PublishingData\">PublishingData</a>&lt;TBlockNum&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.68.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;TBlockNum&gt;: Decode,</span>"],["impl Decode for <a class=\"enum\" href=\"pallet_fragments/enum.SecondarySaleType.html\" title=\"enum pallet_fragments::SecondarySaleType\">SecondarySaleType</a>"],["impl&lt;TAccountId, TBlockNum&gt; Decode for <a class=\"struct\" href=\"pallet_fragments/struct.SecondarySaleData.html\" title=\"struct pallet_fragments::SecondarySaleData\">SecondarySaleData</a>&lt;TAccountId, TBlockNum&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TAccountId: Decode,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.68.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;TBlockNum&gt;: Decode,</span>"],["impl Decode for <a class=\"enum\" href=\"pallet_fragments/enum.SecondarySaleBuyOptions.html\" title=\"enum pallet_fragments::SecondarySaleBuyOptions\">SecondarySaleBuyOptions</a>"],["impl Decode for <a class=\"enum\" href=\"pallet_fragments/enum.FragmentBuyOptions.html\" title=\"enum pallet_fragments::FragmentBuyOptions\">FragmentBuyOptions</a>"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"pallet_fragments/pallet/trait.Config.html\" title=\"trait pallet_fragments::pallet::Config\">Config</a>&gt; Decode for <a class=\"enum\" href=\"pallet_fragments/pallet/enum.Event.html\" title=\"enum pallet_fragments::pallet::Event\">Event</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T::AccountId: Decode,</span>"],["impl&lt;T&gt; Decode for <a class=\"enum\" href=\"pallet_fragments/pallet/enum.Error.html\" title=\"enum pallet_fragments::pallet::Error\">Error</a>&lt;T&gt;"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"pallet_fragments/pallet/trait.Config.html\" title=\"trait pallet_fragments::pallet::Config\">Config</a>&gt; Decode for <a class=\"enum\" href=\"pallet_fragments/pallet/enum.Call.html\" title=\"enum pallet_fragments::pallet::Call\">Call</a>&lt;T&gt;"]],
"pallet_oracle":[["impl Decode for <a class=\"struct\" href=\"pallet_oracle/crypto/struct.Public.html\" title=\"struct pallet_oracle::crypto::Public\">Public</a>"],["impl Decode for <a class=\"struct\" href=\"pallet_oracle/crypto/struct.Signature.html\" title=\"struct pallet_oracle::crypto::Signature\">Signature</a>"],["impl Decode for <a class=\"enum\" href=\"pallet_oracle/enum.OracleProvider.html\" title=\"enum pallet_oracle::OracleProvider\">OracleProvider</a>"],["impl&lt;TPublic, TBlockNumber&gt; Decode for <a class=\"struct\" href=\"pallet_oracle/pallet/struct.OraclePrice.html\" title=\"struct pallet_oracle::pallet::OraclePrice\">OraclePrice</a>&lt;TPublic, TBlockNumber&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TBlockNumber: Decode,<br>&nbsp;&nbsp;&nbsp;&nbsp;TPublic: Decode,</span>"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"pallet_oracle/pallet/trait.Config.html\" title=\"trait pallet_oracle::pallet::Config\">Config</a>&gt; Decode for <a class=\"enum\" href=\"pallet_oracle/pallet/enum.Event.html\" title=\"enum pallet_oracle::pallet::Event\">Event</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T::BlockNumber: Decode,</span>"],["impl&lt;T&gt; Decode for <a class=\"enum\" href=\"pallet_oracle/pallet/enum.Error.html\" title=\"enum pallet_oracle::pallet::Error\">Error</a>&lt;T&gt;"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"pallet_oracle/pallet/trait.Config.html\" title=\"trait pallet_oracle::pallet::Config\">Config</a>&gt; Decode for <a class=\"enum\" href=\"pallet_oracle/pallet/enum.Call.html\" title=\"enum pallet_oracle::pallet::Call\">Call</a>&lt;T&gt;"]],
"pallet_protos":[["impl&lt;TAccountId, TString&gt; Decode for <a class=\"struct\" href=\"pallet_protos/struct.GetProtosParams.html\" title=\"struct pallet_protos::GetProtosParams\">GetProtosParams</a>&lt;TAccountId, TString&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.68.0/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;TString&gt;: Decode,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.68.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;TAccountId&gt;: Decode,</span>"],["impl&lt;TString&gt; Decode for <a class=\"struct\" href=\"pallet_protos/struct.GetGenealogyParams.html\" title=\"struct pallet_protos::GetGenealogyParams\">GetGenealogyParams</a>&lt;TString&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TString: Decode,</span>"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"pallet_protos/pallet/trait.Config.html\" title=\"trait pallet_protos::pallet::Config\">Config</a>&gt; Decode for <a class=\"enum\" href=\"pallet_protos/pallet/enum.Event.html\" title=\"enum pallet_protos::pallet::Event\">Event</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T::AccountId: Decode,</span>"],["impl&lt;T&gt; Decode for <a class=\"enum\" href=\"pallet_protos/pallet/enum.Error.html\" title=\"enum pallet_protos::pallet::Error\">Error</a>&lt;T&gt;"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"pallet_protos/pallet/trait.Config.html\" title=\"trait pallet_protos::pallet::Config\">Config</a>&gt; Decode for <a class=\"enum\" href=\"pallet_protos/pallet/enum.Call.html\" title=\"enum pallet_protos::pallet::Call\">Call</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T::AccountId: UncheckedFrom&lt;T::Hash&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.0/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.68.0/std/primitive.u8.html\">u8</a>]&gt;,</span>"]],
"sp_fragnova":[["impl&lt;TFungibleAsset&gt; Decode for <a class=\"enum\" href=\"sp_fragnova/fragments/enum.Currency.html\" title=\"enum sp_fragnova::fragments::Currency\">Currency</a>&lt;TFungibleAsset&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TFungibleAsset: Decode,</span>"],["impl&lt;TU8Vector, TFungibleAsset&gt; Decode for <a class=\"struct\" href=\"sp_fragnova/fragments/struct.DefinitionMetadata.html\" title=\"struct sp_fragnova::fragments::DefinitionMetadata\">DefinitionMetadata</a>&lt;TU8Vector, TFungibleAsset&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TU8Vector: Decode,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"sp_fragnova/fragments/enum.Currency.html\" title=\"enum sp_fragnova::fragments::Currency\">Currency</a>&lt;TFungibleAsset&gt;: Decode,</span>"],["impl Decode for <a class=\"struct\" href=\"sp_fragnova/fragments/struct.UniqueOptions.html\" title=\"struct sp_fragnova::fragments::UniqueOptions\">UniqueOptions</a>"],["impl&lt;TU8Array, TFungibleAsset, TAccountId, TBlockNum&gt; Decode for <a class=\"struct\" href=\"sp_fragnova/fragments/struct.FragmentDefinition.html\" title=\"struct sp_fragnova::fragments::FragmentDefinition\">FragmentDefinition</a>&lt;TU8Array, TFungibleAsset, TAccountId, TBlockNum&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"sp_fragnova/fragments/struct.DefinitionMetadata.html\" title=\"struct sp_fragnova::fragments::DefinitionMetadata\">DefinitionMetadata</a>&lt;TU8Array, TFungibleAsset&gt;: Decode,<br>&nbsp;&nbsp;&nbsp;&nbsp;TAccountId: Decode,<br>&nbsp;&nbsp;&nbsp;&nbsp;TBlockNum: Decode,</span>"],["impl&lt;TBlockNum&gt; Decode for <a class=\"struct\" href=\"sp_fragnova/fragments/struct.FragmentInstance.html\" title=\"struct sp_fragnova::fragments::FragmentInstance\">FragmentInstance</a>&lt;TBlockNum&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TBlockNum: Decode,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.68.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;TBlockNum&gt;: Decode,</span>"],["impl Decode for <a class=\"enum\" href=\"sp_fragnova/protos/enum.LinkSource.html\" title=\"enum sp_fragnova::protos::LinkSource\">LinkSource</a>"],["impl Decode for <a class=\"enum\" href=\"sp_fragnova/protos/enum.LinkedAsset.html\" title=\"enum sp_fragnova::protos::LinkedAsset\">LinkedAsset</a>"],["impl&lt;TAccountId&gt; Decode for <a class=\"enum\" href=\"sp_fragnova/protos/enum.ProtoOwner.html\" title=\"enum sp_fragnova::protos::ProtoOwner\">ProtoOwner</a>&lt;TAccountId&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TAccountId: Decode,</span>"],["impl&lt;TBlockNumber&gt; Decode for <a class=\"struct\" href=\"sp_fragnova/protos/struct.ProtoPatch.html\" title=\"struct sp_fragnova::protos::ProtoPatch\">ProtoPatch</a>&lt;TBlockNumber&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TBlockNumber: Decode,</span>"],["impl&lt;TContractAddress&gt; Decode for <a class=\"enum\" href=\"sp_fragnova/protos/enum.UsageLicense.html\" title=\"enum sp_fragnova::protos::UsageLicense\">UsageLicense</a>&lt;TContractAddress&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TContractAddress: Decode,</span>"],["impl Decode for <a class=\"enum\" href=\"sp_fragnova/protos/enum.ProtoData.html\" title=\"enum sp_fragnova::protos::ProtoData\">ProtoData</a>"],["impl&lt;TAccountId, TBlockNumber&gt; Decode for <a class=\"struct\" href=\"sp_fragnova/protos/struct.Proto.html\" title=\"struct sp_fragnova::protos::Proto\">Proto</a>&lt;TAccountId, TBlockNumber&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.68.0/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;<a class=\"struct\" href=\"sp_fragnova/protos/struct.ProtoPatch.html\" title=\"struct sp_fragnova::protos::ProtoPatch\">ProtoPatch</a>&lt;TBlockNumber&gt;&gt;: Decode,<br>&nbsp;&nbsp;&nbsp;&nbsp;TBlockNumber: Decode,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"sp_fragnova/protos/enum.UsageLicense.html\" title=\"enum sp_fragnova::protos::UsageLicense\">UsageLicense</a>&lt;TAccountId&gt;: Decode,<br>&nbsp;&nbsp;&nbsp;&nbsp;TAccountId: Decode,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"sp_fragnova/protos/enum.ProtoOwner.html\" title=\"enum sp_fragnova::protos::ProtoOwner\">ProtoOwner</a>&lt;TAccountId&gt;: Decode,</span>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()