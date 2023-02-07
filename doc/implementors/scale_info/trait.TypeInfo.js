(function() {var implementors = {
"fragnova_runtime":[["impl TypeInfo for <a class=\"struct\" href=\"fragnova_runtime/opaque/struct.SessionKeys.html\" title=\"struct fragnova_runtime::opaque::SessionKeys\">SessionKeys</a>"],["impl TypeInfo for <a class=\"struct\" href=\"fragnova_runtime/struct.Runtime.html\" title=\"struct fragnova_runtime::Runtime\">Runtime</a>"],["impl TypeInfo for <a class=\"enum\" href=\"fragnova_runtime/enum.Event.html\" title=\"enum fragnova_runtime::Event\">Event</a>"],["impl TypeInfo for <a class=\"enum\" href=\"fragnova_runtime/enum.OriginCaller.html\" title=\"enum fragnova_runtime::OriginCaller\">OriginCaller</a>"],["impl TypeInfo for <a class=\"enum\" href=\"fragnova_runtime/enum.Call.html\" title=\"enum fragnova_runtime::Call\">Call</a>"]],
"pallet_accounts":[["impl TypeInfo for <a class=\"struct\" href=\"pallet_accounts/crypto/struct.Public.html\" title=\"struct pallet_accounts::crypto::Public\">Public</a>"],["impl TypeInfo for <a class=\"struct\" href=\"pallet_accounts/crypto/struct.Signature.html\" title=\"struct pallet_accounts::crypto::Signature\">Signature</a>"],["impl TypeInfo for <a class=\"enum\" href=\"pallet_accounts/enum.ExternalID.html\" title=\"enum pallet_accounts::ExternalID\">ExternalID</a>"],["impl&lt;TPublic&gt; TypeInfo for <a class=\"struct\" href=\"pallet_accounts/struct.EthLockUpdate.html\" title=\"struct pallet_accounts::EthLockUpdate\">EthLockUpdate</a>&lt;TPublic&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TPublic: TypeInfo + 'static,</span>"],["impl&lt;TBalance, TBlockNum&gt; TypeInfo for <a class=\"struct\" href=\"pallet_accounts/struct.EthLock.html\" title=\"struct pallet_accounts::EthLock\">EthLock</a>&lt;TBalance, TBlockNum&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TBalance: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;TBlockNum: TypeInfo + 'static,</span>"],["impl&lt;TAccountID, TMoment&gt; TypeInfo for <a class=\"struct\" href=\"pallet_accounts/struct.AccountInfo.html\" title=\"struct pallet_accounts::AccountInfo\">AccountInfo</a>&lt;TAccountID, TMoment&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TAccountID: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;TMoment: TypeInfo + 'static,</span>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_accounts/pallet/enum.Event.html\" title=\"enum pallet_accounts::pallet::Event\">Event</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T::AccountId: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;T as Config&gt;::Balance: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;T as Config&gt;::Balance: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;T&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"pallet_accounts/pallet/trait.Config.html\" title=\"trait pallet_accounts::pallet::Config\">Config</a> + 'static,</span>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_accounts/pallet/enum.Error.html\" title=\"enum pallet_accounts::pallet::Error\">Error</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;T&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: 'static,</span>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_accounts/pallet/enum.Call.html\" title=\"enum pallet_accounts::pallet::Call\">Call</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.67.0/std/primitive.tuple.html\">(T,)</a>&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"pallet_accounts/struct.EthLockUpdate.html\" title=\"struct pallet_accounts::EthLockUpdate\">EthLockUpdate</a>&lt;T::Public&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::Signature: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::AccountId: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"pallet_accounts/pallet/trait.Config.html\" title=\"trait pallet_accounts::pallet::Config\">Config</a> + 'static,</span>"]],
"pallet_aliases":[["impl&lt;TAccountId&gt; TypeInfo for <a class=\"enum\" href=\"pallet_aliases/enum.LinkTarget.html\" title=\"enum pallet_aliases::LinkTarget\">LinkTarget</a>&lt;TAccountId&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TAccountId: TypeInfo + 'static,</span>"],["impl&lt;TAccountId, TBlockNum&gt; TypeInfo for <a class=\"struct\" href=\"pallet_aliases/struct.LinkTargetVersioned.html\" title=\"struct pallet_aliases::LinkTargetVersioned\">LinkTargetVersioned</a>&lt;TAccountId, TBlockNum&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"pallet_aliases/enum.LinkTarget.html\" title=\"enum pallet_aliases::LinkTarget\">LinkTarget</a>&lt;TAccountId&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;TBlockNum: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;TAccountId: TypeInfo + 'static,</span>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_aliases/pallet/enum.Event.html\" title=\"enum pallet_aliases::pallet::Event\">Event</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T::AccountId: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;T&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"pallet_aliases/pallet/trait.Config.html\" title=\"trait pallet_aliases::pallet::Config\">Config</a> + 'static,</span>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_aliases/pallet/enum.Error.html\" title=\"enum pallet_aliases::pallet::Error\">Error</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;T&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: 'static,</span>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_aliases/pallet/enum.Call.html\" title=\"enum pallet_aliases::pallet::Call\">Call</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.67.0/std/primitive.tuple.html\">(T,)</a>&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;BoundedVec&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.67.0/std/primitive.u8.html\">u8</a>, &lt;T as <a class=\"trait\" href=\"pallet_aliases/pallet/trait.Config.html\" title=\"trait pallet_aliases::pallet::Config\">Config</a>&gt;::<a class=\"associatedtype\" href=\"pallet_aliases/pallet/trait.Config.html#associatedtype.NameLimit\" title=\"type pallet_aliases::pallet::Config::NameLimit\">NameLimit</a>&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::AccountId: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"pallet_aliases/enum.LinkTarget.html\" title=\"enum pallet_aliases::LinkTarget\">LinkTarget</a>&lt;T::AccountId&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"pallet_aliases/pallet/trait.Config.html\" title=\"trait pallet_aliases::pallet::Config\">Config</a> + 'static,</span>"]],
"pallet_clusters":[["impl TypeInfo for <a class=\"struct\" href=\"pallet_clusters/struct.RoleSetting.html\" title=\"struct pallet_clusters::RoleSetting\">RoleSetting</a>"],["impl TypeInfo for <a class=\"struct\" href=\"pallet_clusters/struct.Role.html\" title=\"struct pallet_clusters::Role\">Role</a>"],["impl&lt;TAccountId&gt; TypeInfo for <a class=\"struct\" href=\"pallet_clusters/struct.Cluster.html\" title=\"struct pallet_clusters::Cluster\">Cluster</a>&lt;TAccountId&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TAccountId: TypeInfo + 'static,</span>"],["impl&lt;TAccountID, TMoment&gt; TypeInfo for <a class=\"struct\" href=\"pallet_clusters/struct.AccountInfo.html\" title=\"struct pallet_clusters::AccountInfo\">AccountInfo</a>&lt;TAccountID, TMoment&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TAccountID: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;TMoment: TypeInfo + 'static,</span>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_clusters/pallet/enum.Event.html\" title=\"enum pallet_clusters::pallet::Event\">Event</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T::AccountId: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;T&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"pallet_clusters/pallet/trait.Config.html\" title=\"trait pallet_clusters::pallet::Config\">Config</a> + 'static,</span>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_clusters/pallet/enum.Error.html\" title=\"enum pallet_clusters::pallet::Error\">Error</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;T&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: 'static,</span>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_clusters/pallet/enum.Call.html\" title=\"enum pallet_clusters::pallet::Call\">Call</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.67.0/std/primitive.tuple.html\">(T,)</a>&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;BoundedVec&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.67.0/std/primitive.u8.html\">u8</a>, T::<a class=\"associatedtype\" href=\"pallet_clusters/pallet/trait.Config.html#associatedtype.NameLimit\" title=\"type pallet_clusters::pallet::Config::NameLimit\">NameLimit</a>&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;BoundedVec&lt;<a class=\"struct\" href=\"pallet_clusters/struct.RoleSetting.html\" title=\"struct pallet_clusters::RoleSetting\">RoleSetting</a>, T::<a class=\"associatedtype\" href=\"pallet_clusters/pallet/trait.Config.html#associatedtype.RoleSettingsLimit\" title=\"type pallet_clusters::pallet::Config::RoleSettingsLimit\">RoleSettingsLimit</a>&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;BoundedVec&lt;T::AccountId, T::<a class=\"associatedtype\" href=\"pallet_clusters/pallet/trait.Config.html#associatedtype.MembersLimit\" title=\"type pallet_clusters::pallet::Config::MembersLimit\">MembersLimit</a>&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;BoundedVec&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.67.0/std/primitive.u8.html\">u8</a>&gt;, T::<a class=\"associatedtype\" href=\"pallet_clusters/pallet/trait.Config.html#associatedtype.RoleSettingsLimit\" title=\"type pallet_clusters::pallet::Config::RoleSettingsLimit\">RoleSettingsLimit</a>&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::AccountId: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"pallet_clusters/pallet/trait.Config.html\" title=\"trait pallet_clusters::pallet::Config\">Config</a> + 'static,</span>"]],
"pallet_detach":[["impl TypeInfo for <a class=\"struct\" href=\"pallet_detach/crypto/struct.Public.html\" title=\"struct pallet_detach::crypto::Public\">Public</a>"],["impl TypeInfo for <a class=\"struct\" href=\"pallet_detach/crypto/struct.Signature.html\" title=\"struct pallet_detach::crypto::Signature\">Signature</a>"],["impl TypeInfo for <a class=\"enum\" href=\"pallet_detach/enum.DetachHash.html\" title=\"enum pallet_detach::DetachHash\">DetachHash</a>"],["impl TypeInfo for <a class=\"enum\" href=\"pallet_detach/enum.DetachCollectionType.html\" title=\"enum pallet_detach::DetachCollectionType\">DetachCollectionType</a>"],["impl TypeInfo for <a class=\"enum\" href=\"pallet_detach/enum.DetachCollection.html\" title=\"enum pallet_detach::DetachCollection\">DetachCollection</a>"],["impl TypeInfo for <a class=\"enum\" href=\"pallet_detach/enum.SupportedChains.html\" title=\"enum pallet_detach::SupportedChains\">SupportedChains</a>"],["impl TypeInfo for <a class=\"struct\" href=\"pallet_detach/struct.DetachRequest.html\" title=\"struct pallet_detach::DetachRequest\">DetachRequest</a>"],["impl&lt;TPublic&gt; TypeInfo for <a class=\"struct\" href=\"pallet_detach/struct.DetachInternalData.html\" title=\"struct pallet_detach::DetachInternalData\">DetachInternalData</a>&lt;TPublic&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TPublic: TypeInfo + 'static,</span>"],["impl TypeInfo for <a class=\"struct\" href=\"pallet_detach/struct.ExportData.html\" title=\"struct pallet_detach::ExportData\">ExportData</a>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_detach/pallet/enum.Event.html\" title=\"enum pallet_detach::pallet::Event\">Event</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;T&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"pallet_detach/pallet/trait.Config.html\" title=\"trait pallet_detach::pallet::Config\">Config</a> + 'static,</span>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_detach/pallet/enum.Error.html\" title=\"enum pallet_detach::pallet::Error\">Error</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;T&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: 'static,</span>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_detach/pallet/enum.Call.html\" title=\"enum pallet_detach::pallet::Call\">Call</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.67.0/std/primitive.tuple.html\">(T,)</a>&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"pallet_detach/struct.DetachInternalData.html\" title=\"struct pallet_detach::DetachInternalData\">DetachInternalData</a>&lt;T::Public&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::Signature: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"pallet_detach/pallet/trait.Config.html\" title=\"trait pallet_detach::pallet::Config\">Config</a> + 'static,</span>"]],
"pallet_fragments":[["impl&lt;TAccountId, TString&gt; TypeInfo for <a class=\"struct\" href=\"pallet_fragments/struct.GetDefinitionsParams.html\" title=\"struct pallet_fragments::GetDefinitionsParams\">GetDefinitionsParams</a>&lt;TAccountId, TString&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;TString&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.67.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;TAccountId&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;TAccountId: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;TString: TypeInfo + 'static,</span>"],["impl&lt;TAccountId, TString&gt; TypeInfo for <a class=\"struct\" href=\"pallet_fragments/struct.GetInstancesParams.html\" title=\"struct pallet_fragments::GetInstancesParams\">GetInstancesParams</a>&lt;TAccountId, TString&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TString: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;TString&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.67.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;TAccountId&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;TAccountId: TypeInfo + 'static,</span>"],["impl&lt;TString&gt; TypeInfo for <a class=\"struct\" href=\"pallet_fragments/struct.GetInstanceOwnerParams.html\" title=\"struct pallet_fragments::GetInstanceOwnerParams\">GetInstanceOwnerParams</a>&lt;TString&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TString: TypeInfo + 'static,</span>"],["impl&lt;TBlockNum&gt; TypeInfo for <a class=\"struct\" href=\"pallet_fragments/struct.PublishingData.html\" title=\"struct pallet_fragments::PublishingData\">PublishingData</a>&lt;TBlockNum&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.67.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;TBlockNum&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;TBlockNum: TypeInfo + 'static,</span>"],["impl TypeInfo for <a class=\"enum\" href=\"pallet_fragments/enum.SecondarySaleType.html\" title=\"enum pallet_fragments::SecondarySaleType\">SecondarySaleType</a>"],["impl&lt;TAccountId, TBlockNum&gt; TypeInfo for <a class=\"struct\" href=\"pallet_fragments/struct.SecondarySaleData.html\" title=\"struct pallet_fragments::SecondarySaleData\">SecondarySaleData</a>&lt;TAccountId, TBlockNum&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TAccountId: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.67.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;TBlockNum&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;TBlockNum: TypeInfo + 'static,</span>"],["impl TypeInfo for <a class=\"enum\" href=\"pallet_fragments/enum.SecondarySaleBuyOptions.html\" title=\"enum pallet_fragments::SecondarySaleBuyOptions\">SecondarySaleBuyOptions</a>"],["impl TypeInfo for <a class=\"enum\" href=\"pallet_fragments/enum.FragmentBuyOptions.html\" title=\"enum pallet_fragments::FragmentBuyOptions\">FragmentBuyOptions</a>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_fragments/pallet/enum.Event.html\" title=\"enum pallet_fragments::pallet::Event\">Event</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T::AccountId: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;T&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"pallet_fragments/pallet/trait.Config.html\" title=\"trait pallet_fragments::pallet::Config\">Config</a> + 'static,</span>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_fragments/pallet/enum.Error.html\" title=\"enum pallet_fragments::pallet::Error\">Error</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;T&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: 'static,</span>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_fragments/pallet/enum.Call.html\" title=\"enum pallet_fragments::pallet::Call\">Call</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.67.0/std/primitive.tuple.html\">(T,)</a>&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"pallet_fragments/struct.DefinitionMetadata.html\" title=\"struct pallet_fragments::DefinitionMetadata\">DefinitionMetadata</a>&lt;BoundedVec&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.67.0/std/primitive.u8.html\">u8</a>, &lt;T as Config&gt;::StringLimit&gt;, T::AssetId&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;BoundedVec&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.67.0/std/primitive.u8.html\">u8</a>, &lt;T as Config&gt;::StringLimit&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.67.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;T::BlockNumber&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;T::Lookup as StaticLookup&gt;::Source: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;BoundedVec&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.67.0/std/primitive.u8.html\">u8</a>, T::DetachAccountLimit&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"pallet_fragments/pallet/trait.Config.html\" title=\"trait pallet_fragments::pallet::Config\">Config</a> + 'static,</span>"]],
"pallet_oracle":[["impl TypeInfo for <a class=\"struct\" href=\"pallet_oracle/crypto/struct.Public.html\" title=\"struct pallet_oracle::crypto::Public\">Public</a>"],["impl TypeInfo for <a class=\"struct\" href=\"pallet_oracle/crypto/struct.Signature.html\" title=\"struct pallet_oracle::crypto::Signature\">Signature</a>"],["impl TypeInfo for <a class=\"enum\" href=\"pallet_oracle/enum.OracleProvider.html\" title=\"enum pallet_oracle::OracleProvider\">OracleProvider</a>"],["impl&lt;TPublic, TBlockNumber&gt; TypeInfo for <a class=\"struct\" href=\"pallet_oracle/pallet/struct.OraclePrice.html\" title=\"struct pallet_oracle::pallet::OraclePrice\">OraclePrice</a>&lt;TPublic, TBlockNumber&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TBlockNumber: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;TPublic: TypeInfo + 'static,</span>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_oracle/pallet/enum.Event.html\" title=\"enum pallet_oracle::pallet::Event\">Event</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T::BlockNumber: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;T&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"pallet_oracle/pallet/trait.Config.html\" title=\"trait pallet_oracle::pallet::Config\">Config</a> + 'static,</span>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_oracle/pallet/enum.Error.html\" title=\"enum pallet_oracle::pallet::Error\">Error</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;T&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: 'static,</span>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_oracle/pallet/enum.Call.html\" title=\"enum pallet_oracle::pallet::Call\">Call</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.67.0/std/primitive.tuple.html\">(T,)</a>&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"pallet_oracle/pallet/struct.OraclePrice.html\" title=\"struct pallet_oracle::pallet::OraclePrice\">OraclePrice</a>&lt;T::Public, T::BlockNumber&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::Signature: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"pallet_oracle/pallet/trait.Config.html\" title=\"trait pallet_oracle::pallet::Config\">Config</a> + 'static,</span>"]],
"pallet_protos":[["impl&lt;TAccountId, TString&gt; TypeInfo for <a class=\"struct\" href=\"pallet_protos/struct.GetProtosParams.html\" title=\"struct pallet_protos::GetProtosParams\">GetProtosParams</a>&lt;TAccountId, TString&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;TString&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.67.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;TAccountId&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;TAccountId: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;TString: TypeInfo + 'static,</span>"],["impl&lt;TString&gt; TypeInfo for <a class=\"struct\" href=\"pallet_protos/struct.GetGenealogyParams.html\" title=\"struct pallet_protos::GetGenealogyParams\">GetGenealogyParams</a>&lt;TString&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TString: TypeInfo + 'static,</span>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_protos/pallet/enum.Event.html\" title=\"enum pallet_protos::pallet::Event\">Event</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T::AccountId: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;T&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"pallet_protos/pallet/trait.Config.html\" title=\"trait pallet_protos::pallet::Config\">Config</a> + 'static,</span>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_protos/pallet/enum.Error.html\" title=\"enum pallet_protos::pallet::Error\">Error</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;T&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: 'static,</span>"],["impl&lt;T&gt; TypeInfo for <a class=\"enum\" href=\"pallet_protos/pallet/enum.Call.html\" title=\"enum pallet_protos::pallet::Call\">Call</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T::AccountId: UncheckedFrom&lt;T::Hash&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.0/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.67.0/std/primitive.u8.html\">u8</a>]&gt; + TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/core/marker/struct.PhantomData.html\" title=\"struct core::marker::PhantomData\">PhantomData</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.67.0/std/primitive.tuple.html\">(T,)</a>&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;BoundedVec&lt;BoundedVec&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.67.0/std/primitive.u8.html\">u8</a>, &lt;T as <a class=\"trait\" href=\"pallet_protos/pallet/trait.Config.html\" title=\"trait pallet_protos::pallet::Config\">Config</a>&gt;::<a class=\"associatedtype\" href=\"pallet_protos/pallet/trait.Config.html#associatedtype.StringLimit\" title=\"type pallet_protos::pallet::Config::StringLimit\">StringLimit</a>&gt;, T::<a class=\"associatedtype\" href=\"pallet_protos/pallet/trait.Config.html#associatedtype.MaxTags\" title=\"type pallet_protos::pallet::Config::MaxTags\">MaxTags</a>&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"pallet_protos/enum.UsageLicense.html\" title=\"enum pallet_protos::UsageLicense\">UsageLicense</a>&lt;T::AccountId&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.67.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"enum\" href=\"pallet_protos/enum.UsageLicense.html\" title=\"enum pallet_protos::UsageLicense\">UsageLicense</a>&lt;T::AccountId&gt;&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.67.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;BoundedVec&lt;BoundedVec&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.67.0/std/primitive.u8.html\">u8</a>, &lt;T as <a class=\"trait\" href=\"pallet_protos/pallet/trait.Config.html\" title=\"trait pallet_protos::pallet::Config\">Config</a>&gt;::<a class=\"associatedtype\" href=\"pallet_protos/pallet/trait.Config.html#associatedtype.StringLimit\" title=\"type pallet_protos::pallet::Config::StringLimit\">StringLimit</a>&gt;, T::<a class=\"associatedtype\" href=\"pallet_protos/pallet/trait.Config.html#associatedtype.MaxTags\" title=\"type pallet_protos::pallet::Config::MaxTags\">MaxTags</a>&gt;&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;BoundedVec&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.67.0/std/primitive.u8.html\">u8</a>, &lt;T as <a class=\"trait\" href=\"pallet_protos/pallet/trait.Config.html\" title=\"trait pallet_protos::pallet::Config\">Config</a>&gt;::<a class=\"associatedtype\" href=\"pallet_protos/pallet/trait.Config.html#associatedtype.StringLimit\" title=\"type pallet_protos::pallet::Config::StringLimit\">StringLimit</a>&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;BoundedVec&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.67.0/std/primitive.u8.html\">u8</a>, T::<a class=\"associatedtype\" href=\"pallet_protos/pallet/trait.Config.html#associatedtype.DetachAccountLimit\" title=\"type pallet_protos::pallet::Config::DetachAccountLimit\">DetachAccountLimit</a>&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"pallet_protos/pallet/trait.Config.html\" title=\"trait pallet_protos::pallet::Config\">Config</a> + 'static,</span>"]],
"sp_fragnova":[["impl TypeInfo for <a class=\"enum\" href=\"sp_fragnova/protos/enum.LinkSource.html\" title=\"enum sp_fragnova::protos::LinkSource\">LinkSource</a>"],["impl TypeInfo for <a class=\"enum\" href=\"sp_fragnova/protos/enum.LinkedAsset.html\" title=\"enum sp_fragnova::protos::LinkedAsset\">LinkedAsset</a>"],["impl&lt;TAccountId&gt; TypeInfo for <a class=\"enum\" href=\"sp_fragnova/protos/enum.ProtoOwner.html\" title=\"enum sp_fragnova::protos::ProtoOwner\">ProtoOwner</a>&lt;TAccountId&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TAccountId: TypeInfo + 'static,</span>"],["impl&lt;TBlockNumber&gt; TypeInfo for <a class=\"struct\" href=\"sp_fragnova/protos/struct.ProtoPatch.html\" title=\"struct sp_fragnova::protos::ProtoPatch\">ProtoPatch</a>&lt;TBlockNumber&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TBlockNumber: TypeInfo + 'static,</span>"],["impl&lt;TContractAddress&gt; TypeInfo for <a class=\"enum\" href=\"sp_fragnova/protos/enum.UsageLicense.html\" title=\"enum sp_fragnova::protos::UsageLicense\">UsageLicense</a>&lt;TContractAddress&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TContractAddress: TypeInfo + 'static,</span>"],["impl TypeInfo for <a class=\"enum\" href=\"sp_fragnova/protos/enum.ProtoData.html\" title=\"enum sp_fragnova::protos::ProtoData\">ProtoData</a>"],["impl&lt;TAccountId, TBlockNumber&gt; TypeInfo for <a class=\"struct\" href=\"sp_fragnova/protos/struct.Proto.html\" title=\"struct sp_fragnova::protos::Proto\">Proto</a>&lt;TAccountId, TBlockNumber&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.67.0/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;<a class=\"struct\" href=\"sp_fragnova/protos/struct.ProtoPatch.html\" title=\"struct sp_fragnova::protos::ProtoPatch\">ProtoPatch</a>&lt;TBlockNumber&gt;&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;TBlockNumber: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"sp_fragnova/protos/enum.UsageLicense.html\" title=\"enum sp_fragnova::protos::UsageLicense\">UsageLicense</a>&lt;TAccountId&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;TAccountId: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"sp_fragnova/protos/enum.ProtoOwner.html\" title=\"enum sp_fragnova::protos::ProtoOwner\">ProtoOwner</a>&lt;TAccountId&gt;: TypeInfo + 'static,</span>"],["impl&lt;TFungibleAsset&gt; TypeInfo for <a class=\"enum\" href=\"sp_fragnova/fragments/enum.Currency.html\" title=\"enum sp_fragnova::fragments::Currency\">Currency</a>&lt;TFungibleAsset&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TFungibleAsset: TypeInfo + 'static,</span>"],["impl&lt;TU8Vector, TFungibleAsset&gt; TypeInfo for <a class=\"struct\" href=\"sp_fragnova/fragments/struct.DefinitionMetadata.html\" title=\"struct sp_fragnova::fragments::DefinitionMetadata\">DefinitionMetadata</a>&lt;TU8Vector, TFungibleAsset&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TU8Vector: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"sp_fragnova/fragments/enum.Currency.html\" title=\"enum sp_fragnova::fragments::Currency\">Currency</a>&lt;TFungibleAsset&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;TFungibleAsset: TypeInfo + 'static,</span>"],["impl TypeInfo for <a class=\"struct\" href=\"sp_fragnova/fragments/struct.UniqueOptions.html\" title=\"struct sp_fragnova::fragments::UniqueOptions\">UniqueOptions</a>"],["impl&lt;TU8Array, TFungibleAsset, TAccountId, TBlockNum&gt; TypeInfo for <a class=\"struct\" href=\"sp_fragnova/fragments/struct.FragmentDefinition.html\" title=\"struct sp_fragnova::fragments::FragmentDefinition\">FragmentDefinition</a>&lt;TU8Array, TFungibleAsset, TAccountId, TBlockNum&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"sp_fragnova/fragments/struct.DefinitionMetadata.html\" title=\"struct sp_fragnova::fragments::DefinitionMetadata\">DefinitionMetadata</a>&lt;TU8Array, TFungibleAsset&gt;: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;TAccountId: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;TBlockNum: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;TU8Array: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;TFungibleAsset: TypeInfo + 'static,</span>"],["impl&lt;TBlockNum&gt; TypeInfo for <a class=\"struct\" href=\"sp_fragnova/fragments/struct.FragmentInstance.html\" title=\"struct sp_fragnova::fragments::FragmentInstance\">FragmentInstance</a>&lt;TBlockNum&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TBlockNum: TypeInfo + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.67.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;TBlockNum&gt;: TypeInfo + 'static,</span>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()